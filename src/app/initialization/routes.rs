use super::handlers;
use axum::{
    Router,
    routing::{get, post},
};

#[rustfmt::skip]
pub fn get_routes() -> Router {
    Router::new()
        .route("/v1/initialization", get(handlers::device_initialization_handler))
        .route("/v1/analytics/gettests", post(handlers::tests_handler))
}
