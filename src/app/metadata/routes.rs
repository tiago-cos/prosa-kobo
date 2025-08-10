use super::handlers;
use crate::app::{AppState, authentication::middleware::extract_token_middleware};
use axum::{Router, middleware::from_fn_with_state, routing::get};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/library/{book_id}/metadata", get(handlers::metadata_handler))
        .layer(from_fn_with_state(state.clone(), extract_token_middleware))
        .with_state(state)
}
