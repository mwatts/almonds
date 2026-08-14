use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::entities::sea_orm_active_enums::ItemType;
use crate::{
    adapters::{meta::RequestMeta, recycle_bin::CreateRecycleBinEntry},
    entities::{recycle_bin, sync_queue},
    error::LunarError,
    utils::{extract_req_meta, js_err, mock_connection, to_js},
};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RecycleBinRepository {
    conn: Arc<DatabaseConnection>,
}

#[async_trait]
pub trait RecycleBinRepositoryExt {
    fn new(conn: Arc<DatabaseConnection>) -> Self;

    async fn store(
        &self,
        payload: &CreateRecycleBinEntry,
        meta: &Option<RequestMeta>,
    ) -> Result<recycle_bin::Model, LunarError>;

    async fn find_all(
        &self,
        meta: &Option<RequestMeta>,
    ) -> Result<Vec<recycle_bin::Model>, LunarError>;

    async fn find_by_id(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<Option<recycle_bin::Model>, LunarError>;

    async fn find_by_item_type(
        &self,
        item_type: &ItemType,
        meta: &Option<RequestMeta>,
    ) -> Result<Vec<recycle_bin::Model>, LunarError>;

    async fn purge(&self, identifier: &Uuid, meta: &Option<RequestMeta>)
    -> Result<(), LunarError>;

    async fn purge_all(&self, meta: &Option<RequestMeta>) -> Result<(), LunarError>;

    async fn extract_unsynced(&self) -> Result<Vec<recycle_bin::Model>, LunarError>;

    async fn clear_synced(&self, identifiers: Vec<String>) -> Result<(), LunarError>;
}

#[async_trait]
impl RecycleBinRepositoryExt for RecycleBinRepository {
    fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { conn }
    }

    async fn store(
        &self,
        payload: &CreateRecycleBinEntry,
        meta: &Option<RequestMeta>,
    ) -> Result<recycle_bin::Model, LunarError> {
        let mut active_model: recycle_bin::ActiveModel = payload.to_owned().into();

        if let Some(meta) = meta {
            active_model.workspace_identifier = Set(Some(meta.workspace_identifier));
        } else {
            return Err(LunarError::DbOperationError(
                "workspace identifier is required".into(),
            ));
        };

        active_model
            .insert(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn find_all(
        &self,
        meta: &Option<RequestMeta>,
    ) -> Result<Vec<recycle_bin::Model>, LunarError> {
        let meta = extract_req_meta(meta)?;

        recycle_bin::Entity::find()
            .filter(recycle_bin::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .order_by_desc(recycle_bin::Column::DeletedAt)
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn find_by_id(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<Option<recycle_bin::Model>, LunarError> {
        let meta = extract_req_meta(meta)?;

        recycle_bin::Entity::find()
            .filter(recycle_bin::Column::Identifier.eq(*identifier))
            .filter(recycle_bin::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn find_by_item_type(
        &self,
        item_type: &ItemType,
        meta: &Option<RequestMeta>,
    ) -> Result<Vec<recycle_bin::Model>, LunarError> {
        let meta = extract_req_meta(meta)?;

        recycle_bin::Entity::find()
            .filter(recycle_bin::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .filter(recycle_bin::Column::ItemType.eq(item_type.to_owned()))
            .order_by_desc(recycle_bin::Column::DeletedAt)
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn purge(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<(), LunarError> {
        let meta = extract_req_meta(meta)?;

        recycle_bin::Entity::delete_many()
            .filter(recycle_bin::Column::Identifier.eq(*identifier))
            .filter(recycle_bin::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .exec(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;
        Ok(())
    }

    async fn purge_all(&self, meta: &Option<RequestMeta>) -> Result<(), LunarError> {
        let meta = extract_req_meta(meta)?;

        recycle_bin::Entity::delete_many()
            .filter(recycle_bin::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .exec(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;
        Ok(())
    }

    async fn extract_unsynced(&self) -> Result<Vec<recycle_bin::Model>, LunarError> {
        let queue_entries = sync_queue::Entity::find()
            .filter(sync_queue::Column::TableName.eq("recycle_bin"))
            .limit(25)
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;

        let identifiers = queue_entries
            .iter()
            .map(|entry| {
                Uuid::parse_str(&entry.record_identifier)
                    .map_err(|err| LunarError::DbOperationError(err.to_string()))
            })
            .collect::<Result<Vec<Uuid>, LunarError>>()?;

        if identifiers.is_empty() {
            return Ok(Vec::new());
        }

        recycle_bin::Entity::find()
            .filter(recycle_bin::Column::Identifier.is_in(identifiers))
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn clear_synced(&self, identifiers: Vec<String>) -> Result<(), LunarError> {
        sync_queue::Entity::delete_many()
            .filter(sync_queue::Column::TableName.eq("recycle_bin"))
            .filter(sync_queue::Column::RecordIdentifier.is_in(identifiers))
            .exec(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;
        Ok(())
    }
}

#[wasm_bindgen]
impl RecycleBinRepository {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm() -> Self {
        Self::new(mock_connection())
    }

    #[wasm_bindgen(js_name = "store")]
    pub async fn store_js(&self, payload: JsValue, meta: JsValue) -> Result<JsValue, JsValue> {
        let payload: CreateRecycleBinEntry =
            serde_wasm_bindgen::from_value(payload).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let model = <Self as RecycleBinRepositoryExt>::store(self, &payload, &meta).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "find_all")]
    pub async fn find_all_js(&self, meta: JsValue) -> Result<JsValue, JsValue> {
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let models = <Self as RecycleBinRepositoryExt>::find_all(self, &meta).await?;
        to_js(&models)
    }

    #[wasm_bindgen(js_name = "find_by_id")]
    pub async fn find_by_id_js(&self, identifier: &str, meta: JsValue) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let model = <Self as RecycleBinRepositoryExt>::find_by_id(self, &id, &meta).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "find_by_item_type")]
    pub async fn find_by_item_type_js(
        &self,
        item_type: JsValue,
        meta: JsValue,
    ) -> Result<JsValue, JsValue> {
        let item_type: ItemType = serde_wasm_bindgen::from_value(item_type).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let models = <Self as RecycleBinRepositoryExt>::find_by_item_type(self, &item_type, &meta)
            .await?;
        to_js(&models)
    }

    #[wasm_bindgen(js_name = "purge")]
    pub async fn purge_js(&self, identifier: &str, meta: JsValue) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        <Self as RecycleBinRepositoryExt>::purge(self, &id, &meta).await?;
        Ok(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(js_name = "purge_all")]
    pub async fn purge_all_js(&self, meta: JsValue) -> Result<JsValue, JsValue> {
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        <Self as RecycleBinRepositoryExt>::purge_all(self, &meta).await?;
        Ok(JsValue::UNDEFINED)
    }
}
