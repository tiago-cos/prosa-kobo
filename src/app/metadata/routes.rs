use super::handlers;
use crate::app::AppState;
use axum::{Router, routing::get};

pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/library/{book_id}/metadata", get(handlers::metadata_handler))
        .with_state(state)
}
