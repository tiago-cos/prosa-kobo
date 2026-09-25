use super::handlers;
use axum::{Router, routing::get};

pub fn get_routes() -> Router {
    Router::new().route("/v1/library/sync", get(handlers::device_sync_handler))
}
