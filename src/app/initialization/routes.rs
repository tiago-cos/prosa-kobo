use crate::app::AppState;
use super::handlers;
use axum::{routing::get, Router};

pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/initialization", get(handlers::device_initialization_handler))
        .with_state(state)
}
