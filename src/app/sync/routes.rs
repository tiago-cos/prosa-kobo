use super::handlers;
use crate::app::{authentication::middleware::extract_token_middleware, AppState};
use axum::{middleware::from_fn_with_state, routing::get, Router};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/library/sync", get(handlers::device_sync_handler))
        .layer(from_fn_with_state(state.clone(), extract_token_middleware))
        .with_state(state)
}
