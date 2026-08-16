use std::sync::Arc;

use axum::{
    routing::{get, patch},
    Router,
};

use crate::{
    handlers::notification::{count_unread, fetch_notifications, mark_read},
    states::AppState,
};

pub(super) fn notification_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(fetch_notifications))
        .route("/unread", get(count_unread))
        .route("/{notification_identifier}", patch(mark_read))
        .with_state(state)
}
