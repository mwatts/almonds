use almond_kernel::entities;
use almond_kernel::entities::sea_orm_active_enums::Tag;
use seaography::async_graphql;
use seaography::CustomInputType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(CustomInputType, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncBookmarkInput {
    pub identifier: Uuid,
    pub title: String,
    pub url: String,
    pub tag: Tag,
    pub created_at: String,
    pub updated_at: String,
    pub workspace_identifier: Option<Uuid>,
}

impl Into<entities::bookmark::Model> for SyncBookmarkInput {
    fn into(self) -> entities::bookmark::Model {
        entities::bookmark::Model {
            identifier: self.identifier,
            title: self.title,
            url: self.url,
            tag: self.tag,
            created_at: self.created_at.parse().unwrap(),
            updated_at: self.updated_at.parse().unwrap(),
            workspace_identifier: self.workspace_identifier,
        }
    }
}
