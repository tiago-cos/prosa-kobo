use super::handlers;
use crate::app::AppState;
use axum::{Router, routing::get};

pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/library/sync", get(handlers::device_sync_handler))
        .with_state(state)
}
