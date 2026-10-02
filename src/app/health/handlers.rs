use super::models::HEALTH_RESPONSE;
use axum::{Json, response::IntoResponse};

pub async fn health_handler() -> impl IntoResponse {
    Json(HEALTH_RESPONSE)
}
