use super::handlers;
use axum::{Router, routing::any};

pub fn get_routes() -> Router {
    Router::new().route("/{*wildcard}", any(handlers::fallback_handler))
}
