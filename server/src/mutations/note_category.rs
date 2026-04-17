use seaography::{
    async_graphql::{self, Context},
    CustomFields,
};
use serde::{Deserialize, Serialize};

use crate::entities;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncNoteCategory;

#[CustomFields]
impl SyncNoteCategory {
    async fn sync_note_category(
        ctx: &Context<'_>,
        input: Vec<entities::note_categories::Model>,
    ) -> async_graphql::Result<bool> {
        unimplemented!()
    }
}
