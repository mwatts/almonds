use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder,
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::entities::sea_orm_active_enums::NotificationType;
use crate::{
    adapters::{meta::RequestMeta, notifications::CreateNotification},
    entities::notifications,
    error::LunarError,
    utils::{extract_req_meta, js_err, mock_connection, to_js},
};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct NotificationRepository {
    conn: Arc<DatabaseConnection>,
}

#[async_trait]
pub trait NotificationRepositoryExt {
    fn new(conn: Arc<DatabaseConnection>) -> Self;

    async fn create(
        &self,
        payload: &CreateNotification,
        meta: &Option<RequestMeta>,
    ) -> Result<notifications::Model, LunarError>;

    async fn find_by_id(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<Option<notifications::Model>, LunarError>;

    async fn find_all(
        &self,
        meta: &Option<RequestMeta>,
    ) -> Result<Vec<notifications::Model>, LunarError>;

    async fn find_by_type(
        &self,
        notification_type: &NotificationType,
        meta: &Option<RequestMeta>,
    ) -> Result<Vec<notifications::Model>, LunarError>;

    async fn mark_as_read(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<notifications::Model, LunarError>;

    async fn delete(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<(), LunarError>;
}

#[async_trait]
impl NotificationRepositoryExt for NotificationRepository {
    fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { conn }
    }

    async fn create(
        &self,
        payload: &CreateNotification,
        meta: &Option<RequestMeta>,
    ) -> Result<notifications::Model, LunarError> {
        let mut active_model: notifications::ActiveModel = payload.to_owned().into();

        let meta = extract_req_meta(meta)?;
        active_model.workspace_identifier = Set(Some(meta.workspace_identifier));

        active_model
            .insert(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn find_by_id(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<Option<notifications::Model>, LunarError> {
        let meta = extract_req_meta(meta)?;

        notifications::Entity::find()
            .filter(notifications::Column::Identifier.eq(*identifier))
            .filter(notifications::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn find_all(
        &self,
        meta: &Option<RequestMeta>,
    ) -> Result<Vec<notifications::Model>, LunarError> {
        let meta = extract_req_meta(meta)?;

        notifications::Entity::find()
            .filter(notifications::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .order_by_desc(notifications::Column::CreatedAt)
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn find_by_type(
        &self,
        notification_type: &NotificationType,
        meta: &Option<RequestMeta>,
    ) -> Result<Vec<notifications::Model>, LunarError> {
        let meta = extract_req_meta(meta)?;

        notifications::Entity::find()
            .filter(notifications::Column::NotificationType.eq(notification_type.to_owned()))
            .filter(notifications::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .order_by_desc(notifications::Column::CreatedAt)
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn mark_as_read(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<notifications::Model, LunarError> {
        let meta = extract_req_meta(meta)?;

        let model = notifications::Entity::find()
            .filter(notifications::Column::Identifier.eq(*identifier))
            .filter(notifications::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?
            .ok_or_else(|| LunarError::NotificationNotFound(identifier.to_string()))?;

        let mut active_model = model.into_active_model();

        active_model.is_read = Set(true);
        active_model.updated_at = Set(Utc::now().fixed_offset());

        active_model
            .update(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn delete(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<(), LunarError> {
        let meta = extract_req_meta(meta)?;

        notifications::Entity::delete_many()
            .filter(notifications::Column::Identifier.eq(*identifier))
            .filter(notifications::Column::WorkspaceIdentifier.eq(meta.workspace_identifier))
            .exec(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;
        Ok(())
    }
}

#[wasm_bindgen]
impl NotificationRepository {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm() -> Self {
        Self::new(mock_connection())
    }

    #[wasm_bindgen(js_name = "create")]
    pub async fn create_js(&self, payload: JsValue, meta: JsValue) -> Result<JsValue, JsValue> {
        let payload: CreateNotification =
            serde_wasm_bindgen::from_value(payload).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let model = <Self as NotificationRepositoryExt>::create(self, &payload, &meta).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "find_by_id")]
    pub async fn find_by_id_js(&self, identifier: &str, meta: JsValue) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let model = <Self as NotificationRepositoryExt>::find_by_id(self, &id, &meta).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "find_all")]
    pub async fn find_all_js(&self, meta: JsValue) -> Result<JsValue, JsValue> {
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let models = <Self as NotificationRepositoryExt>::find_all(self, &meta).await?;
        to_js(&models)
    }

    #[wasm_bindgen(js_name = "find_by_type")]
    pub async fn find_by_type_js(
        &self,
        notification_type: JsValue,
        meta: JsValue,
    ) -> Result<JsValue, JsValue> {
        let notification_type: NotificationType =
            serde_wasm_bindgen::from_value(notification_type).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let models = <Self as NotificationRepositoryExt>::find_by_type(
            self,
            &notification_type,
            &meta,
        )
        .await?;
        to_js(&models)
    }

    #[wasm_bindgen(js_name = "mark_as_read")]
    pub async fn mark_as_read_js(
        &self,
        identifier: &str,
        meta: JsValue,
    ) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let model = <Self as NotificationRepositoryExt>::mark_as_read(self, &id, &meta).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "delete")]
    pub async fn delete_js(&self, identifier: &str, meta: JsValue) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        <Self as NotificationRepositoryExt>::delete(self, &id, &meta).await?;
        Ok(JsValue::UNDEFINED)
    }
}
