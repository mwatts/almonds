use std::sync::Arc;

use axum::{routing::post, Router};

use crate::{handlers::invitation::invite_workspace_member, states::AppState};

pub(super) fn invitation_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/invitations",
            post(invite_workspace_member),
        )
        .with_state(state)
}
