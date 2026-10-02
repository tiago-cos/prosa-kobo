use super::handlers;
use axum::{Router, routing::get};

pub fn get_routes() -> Router {
    Router::new().route("/health", get(handlers::health_handler))
}
