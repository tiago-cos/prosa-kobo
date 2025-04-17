use super::proxy;
use crate::app::AppState;
use axum::{routing::any, Router};

pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/{*wildcard}", any(proxy::proxy_handler))
        .with_state(state)
}
