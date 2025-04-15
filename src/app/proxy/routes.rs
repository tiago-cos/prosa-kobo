use axum::{routing::any, Router};
use crate::app::AppState;
use super::proxy;

pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/{*wildcard}", any(proxy::proxy_handler))
        .with_state(state)
}