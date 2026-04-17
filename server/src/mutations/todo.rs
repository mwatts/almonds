use almond_kernel::entities;
use seaography::{
    async_graphql::{self, Context},
    CustomFields,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncTodo;

#[CustomFields]
impl SyncTodo {
    async fn sync_todo(
        ctx: &Context<'_>,
        input: Vec<entities::todo::Model>,
    ) -> async_graphql::Result<bool> {
        unimplemented!()
    }
}
