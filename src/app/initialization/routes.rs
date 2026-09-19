use super::handlers;
use crate::app::AppState;
use axum::{
    Router,
    routing::{get, post},
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/initialization", get(handlers::device_initialization_handler))
        .route("/v1/analytics/gettests", post(handlers::tests_handler))
        .with_state(state)
}
