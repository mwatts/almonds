use seaography::{
    async_graphql::{self, Context},
    CustomFields,
};
use serde::{Deserialize, Serialize};

use crate::entities;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReminder;

#[CustomFields]
impl SyncReminder {
    async fn sync_reminder(
        ctx: &Context<'_>,
        input: Vec<entities::reminder::Model>,
    ) -> async_graphql::Result<bool> {
        unimplemented!()
    }
}
