use crate::app::{authentication::middleware::extract_token_middleware, AppState};
use super::handlers;
use axum::{middleware::from_fn_with_state, routing::get, Router};

pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/initialization", get(handlers::device_initialization_handler))
        .layer(from_fn_with_state(state.clone(), extract_token_middleware))
        .with_state(state)
}
