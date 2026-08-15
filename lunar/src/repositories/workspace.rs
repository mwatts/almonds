use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QuerySelect,
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::types::EntitySyncResult;
use crate::{
    adapters::{
        meta::RequestMeta,
        workspace::{CreateWorkspace, UpdateWorkspace, hash_password, verify_password},
    },
    entities::{workspaces, sync_queue},
    error::LunarError,
    utils::{extract_req_meta, js_err, mock_connection, to_js},
};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WorkspaceRepository {
    conn: Arc<DatabaseConnection>,
}

#[async_trait]
pub trait WorkspaceRepositoryExt {
    fn new(conn: Arc<DatabaseConnection>) -> Self;

    async fn create_workspace(
        &self,
        workspace: CreateWorkspace,
    ) -> Result<workspaces::Model, LunarError>;

    async fn get_workspace_by_id(&self, id: Uuid) -> Result<workspaces::Model, LunarError>;

    async fn list_workspaces(&self) -> Result<Vec<workspaces::Model>, LunarError>;

    async fn delete_workspace(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<(), LunarError>;

    async fn update_workspace(
        &self,
        identifier: &Uuid,
        payload: UpdateWorkspace,
    ) -> Result<workspaces::Model, LunarError>;

    async fn verify_workspace_password(
        &self,
        identifier: &Uuid,
        password: &str,
    ) -> Result<bool, LunarError>;

    async fn exists(&self, id: &Uuid) -> Result<bool, LunarError>;

    async fn extract_unsynced(&self) -> Result<Vec<workspaces::Model>, LunarError>;

    async fn clear_synced(&self, identifiers: Vec<String>) -> Result<(), LunarError>;
    async fn upsert_many(
        &self,
        models: Vec<workspaces::Model>,
    ) -> Result<Vec<EntitySyncResult>, LunarError>;
}

#[async_trait]
impl WorkspaceRepositoryExt for WorkspaceRepository {
    fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { conn }
    }

    async fn create_workspace(
        &self,
        workspace: CreateWorkspace,
    ) -> Result<workspaces::Model, LunarError> {
        let active_model: workspaces::ActiveModel = workspace.into();
        active_model
            .insert(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn get_workspace_by_id(&self, id: Uuid) -> Result<workspaces::Model, LunarError> {
        workspaces::Entity::find()
            .filter(workspaces::Column::Identifier.eq(id))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
            .and_then(|opt| {
                opt.ok_or_else(|| {
                    LunarError::DbOperationError(format!("Workspace with id {} not found", id))
                })
            })
    }

    async fn list_workspaces(&self) -> Result<Vec<workspaces::Model>, LunarError> {
        workspaces::Entity::find()
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn delete_workspace(
        &self,
        identifier: &Uuid,
        meta: &Option<RequestMeta>,
    ) -> Result<(), LunarError> {
        let _meta = extract_req_meta(meta)?;

        let model = workspaces::Entity::find()
            .filter(workspaces::Column::Identifier.eq(*identifier))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?
            .ok_or_else(|| LunarError::DbOperationError("workspace not found".to_string()))?;

        if model.is_default {
            return Err(LunarError::DbOperationError(
                "Cannot delete the default workspace".to_string(),
            ));
        }

        let _payload = serde_json::to_string(&model)
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;

        //TODO: Consider moving this logic to a service layer if it becomes more complex or if we need to handle related entities (e.g., todos, bookmarks) in the recycle bin entry. For now, it serves the purpose of keeping a record of deleted workspaces.
        // RecycleBinRepository::new(self.conn.clone())
        //     .store(
        //         &CreateRecycleBinEntry {
        //             item_id: model.identifier,
        //             item_type: ItemType::Workspace,
        //             workspace_identifier: None,
        //             payload,
        //         },
        //         &Some(meta.clone()),
        //     )
        //     .await?;

        let result = workspaces::Entity::delete_by_id(*identifier)
            .exec(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;

        log::info!("{:#?}", result);
        Ok(())
    }

    async fn update_workspace(
        &self,
        identifier: &Uuid,
        payload: UpdateWorkspace,
    ) -> Result<workspaces::Model, LunarError> {
        // If promoting to default, demote all others first
        if payload.is_default == Some(true) {
            workspaces::Entity::update_many()
                .col_expr(workspaces::Column::IsDefault, Expr::value(false))
                .exec(self.conn.as_ref())
                .await
                .map_err(|err| LunarError::DbOperationError(err.to_string()))?;
        }

        let model = self.get_workspace_by_id(*identifier).await?;
        let mut active: workspaces::ActiveModel = model.into();

        if let Some(name) = payload.name {
            active.name = Set(name);
        }
        if let Some(description) = payload.description {
            active.description = Set(description);
        }
        if let Some(is_default) = payload.is_default {
            active.is_default = Set(is_default);
        }
        if let Some(is_hidden) = payload.is_hidden {
            active.is_hidden = Set(is_hidden);
        }
        if let Some(is_secured) = payload.is_secured {
            active.is_secured = Set(is_secured);
            // When disabling security, clear the hash
            if !is_secured {
                active.password_hash = Set(None);
            }
        }
        if let Some(password) = payload.password {
            if password.is_empty() {
                active.password_hash = Set(None);
            } else {
                let hash = hash_password(&password)
                    .map_err(|e| LunarError::DbOperationError(e.to_string()))?;
                active.password_hash = Set(Some(hash));
            }
        }
        active.updated_at = Set(Utc::now().fixed_offset());

        active
            .update(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn verify_workspace_password(
        &self,
        identifier: &Uuid,
        password: &str,
    ) -> Result<bool, LunarError> {
        let model = self.get_workspace_by_id(*identifier).await?;
        if !model.is_secured {
            return Ok(true);
        }
        match model.password_hash {
            Some(ref hash) => verify_password(password, hash)
                .map_err(|e| LunarError::DbOperationError(e.to_string())),
            None => Ok(false),
        }
    }

    async fn exists(&self, id: &Uuid) -> Result<bool, LunarError> {
        let result = self.get_workspace_by_id(id.to_owned()).await.ok();
        Ok(result.is_some())
    }

    async fn extract_unsynced(&self) -> Result<Vec<workspaces::Model>, LunarError> {
        let queue_entries = sync_queue::Entity::find()
            .filter(sync_queue::Column::TableName.eq("workspaces"))
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

        workspaces::Entity::find()
            .filter(workspaces::Column::Identifier.is_in(identifiers))
            .all(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn clear_synced(&self, identifiers: Vec<String>) -> Result<(), LunarError> {
        sync_queue::Entity::delete_many()
            .filter(sync_queue::Column::TableName.eq("workspaces"))
            .filter(sync_queue::Column::RecordIdentifier.is_in(identifiers))
            .exec(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?;
        Ok(())
    }
    async fn upsert_many(
        &self,
        models: Vec<workspaces::Model>,
    ) -> Result<Vec<EntitySyncResult>, LunarError> {
        let mut sync_results: Vec<EntitySyncResult> = Vec::new();
        for chunk in models.chunks(20) {
            let futures: Vec<_> = chunk
                .iter()
                .map(|model| {
                    let conn = self.conn.clone();
                    let model = model.clone();
                    async move {
                        let identifier = model.identifier.to_string();
                        let op_result: Result<(), LunarError> = async {
                            let exists = workspaces::Entity::find()
                                .filter(workspaces::Column::Identifier.eq(model.identifier))
                                .one(conn.as_ref())
                                .await
                                .map_err(|err| LunarError::DbOperationError(err.to_string()))?
                                .is_some();

                            let active_model: workspaces::ActiveModel = model.into();

                            if exists {
                                active_model.update(conn.as_ref()).await.map_err(|err| {
                                    LunarError::DbOperationError(err.to_string())
                                })?;
                            } else {
                                active_model.insert(conn.as_ref()).await.map_err(|err| {
                                    LunarError::DbOperationError(err.to_string())
                                })?;
                            }
                            Ok(())
                        }
                        .await;
                        EntitySyncResult {
                            identifier,
                            success: op_result.is_ok(),
                            error_message: op_result.err().map(|e| e.to_string()),
                        }
                    }
                })
                .collect();

            let chunk_results = futures::future::join_all(futures).await;
            sync_results.extend(chunk_results);
        }
        Ok(sync_results)
    }
}
#[wasm_bindgen]
impl WorkspaceRepository {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm() -> Self {
        Self::new(mock_connection())
    }

    #[wasm_bindgen(js_name = "create_workspace")]
    pub async fn create_workspace_js(&self, payload: JsValue) -> Result<JsValue, JsValue> {
        let payload: CreateWorkspace = serde_wasm_bindgen::from_value(payload).map_err(js_err)?;
        let model = <Self as WorkspaceRepositoryExt>::create_workspace(self, payload).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "get_workspace_by_id")]
    pub async fn get_workspace_by_id_js(&self, identifier: &str) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let model = <Self as WorkspaceRepositoryExt>::get_workspace_by_id(self, id).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "list_workspaces")]
    pub async fn list_workspaces_js(&self) -> Result<JsValue, JsValue> {
        let models = <Self as WorkspaceRepositoryExt>::list_workspaces(self).await?;
        to_js(&models)
    }

    #[wasm_bindgen(js_name = "delete_workspace")]
    pub async fn delete_workspace_js(
        &self,
        identifier: &str,
        meta: JsValue,
    ) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let meta: Option<RequestMeta> = serde_wasm_bindgen::from_value(meta).map_err(js_err)?;
        <Self as WorkspaceRepositoryExt>::delete_workspace(self, &id, &meta).await?;
        Ok(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(js_name = "update_workspace")]
    pub async fn update_workspace_js(
        &self,
        identifier: &str,
        payload: JsValue,
    ) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let payload: UpdateWorkspace = serde_wasm_bindgen::from_value(payload).map_err(js_err)?;
        let model = <Self as WorkspaceRepositoryExt>::update_workspace(self, &id, payload).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "verify_workspace_password")]
    pub async fn verify_workspace_password_js(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<bool, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        <Self as WorkspaceRepositoryExt>::verify_workspace_password(self, &id, password)
            .await
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = "exists")]
    pub async fn exists_js(&self, identifier: &str) -> Result<bool, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        <Self as WorkspaceRepositoryExt>::exists(self, &id).await.map_err(JsValue::from)
    }
}
