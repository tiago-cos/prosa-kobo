use super::handlers;
use crate::app::AppState;
use axum::{
    routing::{get, post},
    Router,
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/devices/unlinked", get(handlers::get_unlinked_devices_handler))
        .route("/devices/linked", get(handlers::get_linked_devices_handler))
        .route("/devices/link", post(handlers::link_device_handler))
        .route("/devices/unlink", post(handlers::unlink_device_handler))
        .route("/v1/auth/device", post(handlers::device_auth_handler))
        .route("/v1/auth/refresh", post(handlers::refresh_token_handler))
        .with_state(state)
}
