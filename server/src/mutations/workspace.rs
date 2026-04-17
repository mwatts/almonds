use seaography::{
    async_graphql::{self, Context},
    CustomFields,
};
use serde::{Deserialize, Serialize};

use almond_kernel::entities;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncWorkspace;

#[CustomFields]
impl SyncWorkspace {
    async fn sync_workspace(
        ctx: &Context<'_>,
        input: Vec<entities::workspaces::Model>,
    ) -> async_graphql::Result<bool> {
        unimplemented!()
    }
}
