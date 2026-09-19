use super::handlers;
use crate::app::{
    AppState,
    authentication::middleware::extract_prosa_token_middleware,
    authorization::devices::{
        can_link_device, can_read_unlinked_devices, can_search_linked_devices, can_unlink_device,
    },
};
use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, post},
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/devices/unlinked", get(handlers::get_unlinked_devices_handler)
            .route_layer(from_fn(can_read_unlinked_devices))
        )
        .route("/devices/linked", get(handlers::get_linked_devices_handler)
            .route_layer(from_fn(can_search_linked_devices))
        )
        .route("/devices/linked", post(handlers::link_device_handler)
            .route_layer(from_fn(can_link_device))
        )
        .route("/devices/linked/{device_id}", delete(handlers::unlink_device_handler)
            .route_layer(from_fn_with_state(state.clone(), can_unlink_device))
        )
        .layer(from_fn(extract_prosa_token_middleware))
        .route("/v1/auth/device", post(handlers::device_auth_handler))
        .route("/v1/auth/refresh", post(handlers::refresh_token_handler))
        .with_state(state)
}
