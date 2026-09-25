use super::proxy;
use axum::{Router, routing::any};

pub fn get_routes() -> Router {
    Router::new().route("/{*wildcard}", any(proxy::proxy_handler))
}
