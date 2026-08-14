use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QuerySelect,
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::repositories::{
    prelude::WorkspaceRepositoryExt,
    workspace::WorkspaceRepository,
    workspace_manager::{DuplicateRecord, RecordExistInWorkspace, TransferRecord},
};
use crate::{
    adapters::{
        meta::RequestMeta,
        workspace_preferences::{CreateUserPreference, UpdateUserPreference},
    },
    entities::{workspace_preferences, sync_queue},
    error::LunarError,
    utils::{extract_req_meta, js_err, mock_connection, to_js},
};

#[wasm_bindgen]
pub struct WorkspacePreferenceRepository {
    conn: Arc<DatabaseConnection>,
    workspace_repository: WorkspaceRepository,
}

#[async_trait]
pub trait WorkspacePreferenceRepositoryExt {
    fn new(conn: Arc<DatabaseConnection>) -> Self;

    async fn create(
        &self,
        payload: &CreateUserPreference,
        meta: &Option<RequestMeta>,
    ) -> Result<workspace_preferences::Model, LunarError>;

    async fn get(
        &self,
        meta: &Option<RequestMeta>,
    ) -> Result<Option<workspace_preferences::Model>, LunarError>;

    async fn update(
        &self,
        identifier: &Uuid,
        payload: &UpdateUserPreference,
        meta: &Option<RequestMeta>,
    ) -> Result<workspace_preferences::Model, LunarError>;

    async fn extract_unsynced(&self) -> Result<Vec<workspace_preferences::Model>, LunarError>;

    async fn clear_synced(&self, identifiers: Vec<String>) -> Result<(), LunarError>;
}

#[async_trait]
impl WorkspacePreferenceRepositoryExt for WorkspacePreferenceRepository {
    fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self {
            conn: conn.clone(),
            workspace_repository: WorkspaceRepository::new(conn.clone()),
        }
    }

    async fn create(
        &self,
        payload: &CreateUserPreference,
        meta: &Option<RequestMeta>,
    ) -> Result<workspace_preferences::Model, LunarError> {
        let mut active_model: workspace_preferences::ActiveModel = payload.to_owned().into();

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

    async fn get(
        &self,
        meta: &Option<RequestMeta>,
    ) -> Result<Option<workspace_preferences::Model>, LunarError> {
        let meta = extract_req_meta(meta)?;

        workspace_preferences::Entity::find()
            .filter(
                workspace_preferences::Column::WorkspaceIdentifier.eq(meta.workspace_identifier),
            )
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn update(
        &self,
        identifier: &Uuid,
        payload: &UpdateUserPreference,
        meta: &Option<RequestMeta>,
    ) -> Result<workspace_preferences::Model, LunarError> {
        let meta = extract_req_meta(meta)?;

        let model = workspace_preferences::Entity::find()
            .filter(workspace_preferences::Column::Identifier.eq(*identifier))
            .filter(
                workspace_preferences::Column::WorkspaceIdentifier.eq(meta.workspace_identifier),
            )
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?
            .ok_or_else(|| {
                LunarError::DbOperationError("user preference not found".to_string())
            })?;

        let mut active_model = model.into_active_model();

        if let Some(first_name) = &payload.first_name {
            active_model.first_name = Set(first_name.clone());
        }
        if let Some(last_name) = &payload.last_name {
            active_model.last_name = Set(last_name.clone());
        }

        active_model.updated_at = Set(Utc::now().fixed_offset());

        active_model
            .update(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn extract_unsynced(&self) -> Result<Vec<workspace_preferences::Model>, LunarError> {
        let queue_entries = sync_queue::Entity::find()
            .filter(sync_queue::Column::TableName.eq("workspace_preferences"))
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

        workspace_preferences::Entity::find()
            .filter(workspace_preferences::Column::Identifier.is_in(identifiers))
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn clear_synced(&self, identifiers: Vec<String>) -> Result<(), LunarError> {
        sync_queue::Entity::delete_many()
            .filter(sync_queue::Column::TableName.eq("workspace_preferences"))
            .filter(sync_queue::Column::RecordIdentifier.is_in(identifiers))
            .exec(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl TransferRecord for WorkspacePreferenceRepository {
    async fn transfer_record(
        &self,
        record_identifier: &Uuid,
        previous_workspace_identifier: &Uuid,
        target_workspace_identifier: &Uuid,
    ) -> Result<(), LunarError> {
        let (prev_exists_res, target_exists_res) = tokio::join!(
            self.workspace_repository
                .exists(previous_workspace_identifier),
            self.workspace_repository
                .exists(target_workspace_identifier),
        );

        let prev_exists = prev_exists_res?;
        let target_exists = target_exists_res?;

        if !prev_exists {
            return Err(LunarError::WorkspaceNotFound(
                previous_workspace_identifier.to_string(),
            ));
        }

        if !target_exists {
            return Err(LunarError::WorkspaceNotFound(
                target_workspace_identifier.to_string(),
            ));
        }

        if !self
            .record_exists_in_workspace(record_identifier, previous_workspace_identifier)
            .await?
        {
            return Err(LunarError::BookmarkNotFound(record_identifier.to_string()));
        }

        let Some(record) = workspace_preferences::Entity::find()
            .filter(workspace_preferences::Column::Identifier.eq(*record_identifier))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?
        else {
            return Err(LunarError::BookmarkNotFound(record_identifier.to_string()));
        };

        let mut active_model = record.into_active_model();

        active_model.updated_at = Set(Utc::now().fixed_offset());
        active_model.workspace_identifier = Set(Some(*target_workspace_identifier));

        active_model
            .update(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl RecordExistInWorkspace for WorkspacePreferenceRepository {
    async fn record_exists_in_workspace(
        &self,
        record_identifier: &Uuid,
        workspace_identifier: &Uuid,
    ) -> Result<bool, LunarError> {
        let record = workspace_preferences::Entity::find()
            .filter(workspace_preferences::Column::Identifier.eq(*record_identifier))
            .filter(workspace_preferences::Column::WorkspaceIdentifier.eq(*workspace_identifier))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;

        Ok(record.is_some())
    }
}

#[async_trait::async_trait]
impl DuplicateRecord for WorkspacePreferenceRepository {
    async fn duplicate_record(
        &self,
        record_identifier: &Uuid,
        previous_workspace_identifier: &Uuid,
        target_workspace_identifier: &Uuid,
    ) -> Result<(), LunarError> {
        let (prev_exists_res, target_exists_res) = tokio::join!(
            self.workspace_repository
                .exists(previous_workspace_identifier),
            self.workspace_repository
                .exists(target_workspace_identifier),
        );

        let prev_exists = prev_exists_res?;
        let target_exists = target_exists_res?;

        if !prev_exists {
            return Err(LunarError::WorkspaceNotFound(
                previous_workspace_identifier.to_string(),
            ));
        }

        if !target_exists {
            return Err(LunarError::WorkspaceNotFound(
                target_workspace_identifier.to_string(),
            ));
        }

        let Some(record) = workspace_preferences::Entity::find()
            .filter(workspace_preferences::Column::Identifier.eq(*record_identifier))
            .filter(
                workspace_preferences::Column::WorkspaceIdentifier
                    .eq(*previous_workspace_identifier),
            )
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?
        else {
            return Err(LunarError::BookmarkNotFound(record_identifier.to_string()));
        };

        let mut new_record = record.into_active_model();

        new_record.identifier = Set(Uuid::new_v4());
        new_record.workspace_identifier = Set(Some(*target_workspace_identifier));
        new_record.created_at = Set(Utc::now().fixed_offset());
        new_record.updated_at = Set(Utc::now().fixed_offset());

        new_record
            .insert(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;

        Ok(())
    }
}

#[wasm_bindgen]
impl WorkspacePreferenceRepository {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm() -> Self {
        Self::new(mock_connection())
    }

    #[wasm_bindgen(js_name = "create")]
    pub async fn create_js(&self, payload: JsValue, meta: JsValue) -> Result<JsValue, JsValue> {
        let payload: CreateUserPreference =
            serde_wasm_bindgen::from_value(payload).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let model = <Self as WorkspacePreferenceRepositoryExt>::create(self, &payload, &meta)
            .await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "get")]
    pub async fn get_js(&self, meta: JsValue) -> Result<JsValue, JsValue> {
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let model = <Self as WorkspacePreferenceRepositoryExt>::get(self, &meta).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "update")]
    pub async fn update_js(
        &self,
        identifier: &str,
        payload: JsValue,
        meta: JsValue,
    ) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let payload: UpdateUserPreference =
            serde_wasm_bindgen::from_value(payload).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        let model =
            <Self as WorkspacePreferenceRepositoryExt>::update(self, &id, &payload, &meta).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "transfer_record")]
    pub async fn transfer_record_js(
        &self,
        record_identifier: &str,
        previous_workspace_identifier: &str,
        target_workspace_identifier: &str,
    ) -> Result<JsValue, JsValue> {
        let record_identifier = Uuid::parse_str(record_identifier).map_err(js_err)?;
        let previous_workspace_identifier =
            Uuid::parse_str(previous_workspace_identifier).map_err(js_err)?;
        let target_workspace_identifier =
            Uuid::parse_str(target_workspace_identifier).map_err(js_err)?;
        <Self as TransferRecord>::transfer_record(
            self,
            &record_identifier,
            &previous_workspace_identifier,
            &target_workspace_identifier,
        )
        .await?;
        Ok(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(js_name = "duplicate_record")]
    pub async fn duplicate_record_js(
        &self,
        record_identifier: &str,
        previous_workspace_identifier: &str,
        target_workspace_identifier: &str,
    ) -> Result<JsValue, JsValue> {
        let record_identifier = Uuid::parse_str(record_identifier).map_err(js_err)?;
        let previous_workspace_identifier =
            Uuid::parse_str(previous_workspace_identifier).map_err(js_err)?;
        let target_workspace_identifier =
            Uuid::parse_str(target_workspace_identifier).map_err(js_err)?;
        <Self as DuplicateRecord>::duplicate_record(
            self,
            &record_identifier,
            &previous_workspace_identifier,
            &target_workspace_identifier,
        )
        .await?;
        Ok(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(js_name = "record_exists_in_workspace")]
    pub async fn record_exists_in_workspace_js(
        &self,
        record_identifier: &str,
        workspace_identifier: &str,
    ) -> Result<bool, JsValue> {
        let record_identifier = Uuid::parse_str(record_identifier).map_err(js_err)?;
        let workspace_identifier = Uuid::parse_str(workspace_identifier).map_err(js_err)?;
        <Self as RecordExistInWorkspace>::record_exists_in_workspace(
            self,
            &record_identifier,
            &workspace_identifier,
        )
        .await
        .map_err(JsValue::from)
    }
}
