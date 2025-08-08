use super::handlers;
use crate::app::{AppState, authentication::middleware::extract_token_middleware};
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/initialization", get(handlers::device_initialization_handler)
            .route_layer(from_fn_with_state(state.clone(), extract_token_middleware))
        )
        .route("/v1/analytics/gettests", post(handlers::tests_handler))
        .with_state(state)
}
