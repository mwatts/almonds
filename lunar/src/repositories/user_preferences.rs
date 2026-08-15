use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter,
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::types::EntitySyncResult;
use crate::{
    adapters::user_preferences::{CreateUserPreferences, UpdateUserPreferences},
    entities::user_preferences,
    error::LunarError,
    utils::{js_err, mock_connection, to_js},
};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct UserPreferencesRepository {
    conn: Arc<DatabaseConnection>,
}

#[async_trait]
pub trait UserPreferencesRepositoryExt {
    fn new(conn: Arc<DatabaseConnection>) -> Self;

    async fn create(
        &self,
        payload: &CreateUserPreferences,
    ) -> Result<user_preferences::Model, LunarError>;

    async fn get_by_identifier(
        &self,
        identifier: &Uuid,
    ) -> Result<Option<user_preferences::Model>, LunarError>;

    async fn update(
        &self,
        identifier: &Uuid,
        payload: &UpdateUserPreferences,
    ) -> Result<user_preferences::Model, LunarError>;
    async fn upsert_many(
        &self,
        models: Vec<user_preferences::Model>,
    ) -> Result<Vec<EntitySyncResult>, LunarError>;
}

#[async_trait]
impl UserPreferencesRepositoryExt for UserPreferencesRepository {
    fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { conn }
    }

    async fn create(
        &self,
        payload: &CreateUserPreferences,
    ) -> Result<user_preferences::Model, LunarError> {
        let active_model: user_preferences::ActiveModel = payload.to_owned().into();
        active_model
            .insert(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn get_by_identifier(
        &self,
        identifier: &Uuid,
    ) -> Result<Option<user_preferences::Model>, LunarError> {
        user_preferences::Entity::find()
            .filter(user_preferences::Column::Identifier.eq(*identifier))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }

    async fn update(
        &self,
        identifier: &Uuid,
        payload: &UpdateUserPreferences,
    ) -> Result<user_preferences::Model, LunarError> {
        let model = user_preferences::Entity::find()
            .filter(user_preferences::Column::Identifier.eq(*identifier))
            .one(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))?
            .ok_or_else(|| {
                LunarError::DbOperationError("user preferences not found".to_string())
            })?;

        let mut active_model = model.into_active_model();

        if let Some(first_name) = &payload.master_first_name {
            active_model.master_first_name = Set(first_name.clone());
        }
        if let Some(last_name) = &payload.master_last_name {
            active_model.master_last_name = Set(last_name.clone());
        }
        if let Some(email) = &payload.master_email {
            active_model.master_email = Set(email.clone());
        }
        active_model.updated_at = Set(Utc::now().fixed_offset());

        active_model
            .update(self.conn.as_ref())
            .await
            .map_err(|err| LunarError::DbOperationError(err.to_string()))
    }
    async fn upsert_many(
        &self,
        models: Vec<user_preferences::Model>,
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
                            let exists = user_preferences::Entity::find()
                                .filter(user_preferences::Column::Identifier.eq(model.identifier))
                                .one(conn.as_ref())
                                .await
                                .map_err(|err| LunarError::DbOperationError(err.to_string()))?
                                .is_some();

                            let active_model = model.into_active_model();

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
impl UserPreferencesRepository {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm() -> Self {
        Self::new(mock_connection())
    }

    #[wasm_bindgen(js_name = "create")]
    pub async fn create_js(&self, payload: JsValue) -> Result<JsValue, JsValue> {
        let payload: CreateUserPreferences =
            serde_wasm_bindgen::from_value(payload).map_err(js_err)?;
        let model = <Self as UserPreferencesRepositoryExt>::create(self, &payload).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "get_by_identifier")]
    pub async fn get_by_identifier_js(&self, identifier: &str) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let model = <Self as UserPreferencesRepositoryExt>::get_by_identifier(self, &id).await?;
        to_js(&model)
    }

    #[wasm_bindgen(js_name = "update")]
    pub async fn update_js(
        &self,
        identifier: &str,
        payload: JsValue,
    ) -> Result<JsValue, JsValue> {
        let id = Uuid::parse_str(identifier).map_err(js_err)?;
        let payload: UpdateUserPreferences =
            serde_wasm_bindgen::from_value(payload).map_err(js_err)?;
        let model = <Self as UserPreferencesRepositoryExt>::update(self, &id, &payload).await?;
        to_js(&model)
    }
}
