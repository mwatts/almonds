use seaography::{
    async_graphql::{self, Context},
    CustomFields,
};
use serde::{Deserialize, Serialize};

use crate::entities;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncNote;

#[CustomFields]
impl SyncNote {
    async fn sync_note(
        ctx: &Context<'_>,
        input: Vec<entities::notes::Model>,
    ) -> async_graphql::Result<bool> {
        unimplemented!()
    }
}
