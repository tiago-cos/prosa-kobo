use super::handlers;
use axum::{Router, routing::get};

pub fn get_routes() -> Router {
    Router::new().route("/v1/library/{book_id}/metadata", get(handlers::metadata_handler))
}
