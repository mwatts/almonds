use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    adapters::{
        invitation::{InviteWorkspaceMemberRequest, InviteWorkspaceMemberResponse},
        request::AuthenticatedRequest,
    },
    errors::app_error::AppError,
    states::AppState,
};

pub async fn invite_workspace_member(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    AuthenticatedRequest { claims, data }: AuthenticatedRequest<InviteWorkspaceMemberRequest>,
) -> Result<(StatusCode, Json<InviteWorkspaceMemberResponse>), AppError> {
    let requester_id = claims.user_identifier;

    let response = state
        .services
        .invitation_service
        .invite_member(workspace_id, &requester_id, &data)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}
