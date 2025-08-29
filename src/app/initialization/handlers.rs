use super::{models::TestRequest, service};
use crate::app::authentication::AuthToken;
use axum::{Extension, Json, response::IntoResponse};
use axum_extra::extract::Host;

pub async fn device_initialization_handler(
    Host(host): Host,
    Extension(token): Extension<AuthToken>,
) -> impl IntoResponse {
    Json(service::generate_initialization_response(&host, &token.device_id))
}

pub async fn tests_handler(Json(request): Json<TestRequest>) -> impl IntoResponse {
    Json(service::generate_tests_response(&request.test_key))
}
