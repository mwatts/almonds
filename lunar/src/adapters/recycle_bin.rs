use chrono::Utc;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{self, recycle_bin::ActiveModel, sea_orm_active_enums::ItemType};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "recycle_bin.ts")]
pub struct CreateRecycleBinEntry {
    pub item_id: Uuid,
    pub item_type: ItemType,
    pub payload: String,
    pub workspace_identifier: Option<Uuid>,
}

impl Into<entities::recycle_bin::ActiveModel> for CreateRecycleBinEntry {
    fn into(self) -> entities::recycle_bin::ActiveModel {
        ActiveModel {
            identifier: Set(Uuid::new_v4()),
            item_id: Set(self.item_id),
            item_type: Set(self.item_type),
            payload: Set(self.payload),
            workspace_identifier: Set(self.workspace_identifier),
            deleted_at: Set(Utc::now().fixed_offset()),
        }
    }
}
