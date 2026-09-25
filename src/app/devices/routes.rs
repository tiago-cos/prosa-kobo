use super::handlers;
use crate::app::{
    authentication::middleware::extract_prosa_token_middleware,
    authorization::devices::{can_link_device, can_search_linked_devices, can_unlink_device},
};
use axum::{
    Router,
    middleware::from_fn,
    routing::{delete, get, post},
};

#[rustfmt::skip]
pub fn get_routes() -> Router {
    Router::new()
        .route("/devices/linked", get(handlers::get_linked_devices_handler)
            .route_layer(from_fn(can_search_linked_devices))
        )
        .route("/devices/linked", post(handlers::link_device_handler)
            .route_layer(from_fn(can_link_device))
        )
        .route("/devices/linked/{device_id}", delete(handlers::unlink_device_handler)
            .route_layer(from_fn(can_unlink_device))
        )
        .layer(from_fn(extract_prosa_token_middleware))
}
