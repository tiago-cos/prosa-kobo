use super::proxy;
use crate::app::AppState;
use axum::{Router, routing::any};

pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/{*wildcard}", any(proxy::proxy_handler))
        .with_state(state)
}
