use super::handlers;
use crate::app::{authentication::middleware::extract_token_middleware, AppState};
use axum::{
    middleware::from_fn_with_state,
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
        .route("/devices/auth", post(handlers::device_auth_handler))
        .route("/devices/auth/refresh", post(handlers::refresh_token_handler))
        .route("/tmp", get(handlers::tmp)
            .route_layer(from_fn_with_state(state.clone(), extract_token_middleware))
        )
        .with_state(state)
}
