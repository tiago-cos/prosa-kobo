use super::service;
use axum::{
    response::IntoResponse,
    Json,
};
use axum_extra::extract::Host;

pub async fn device_initialization_handler(
    host: Host,
) -> impl IntoResponse {
    Json(service::generate_initialization_response(&host.0).await)
}
