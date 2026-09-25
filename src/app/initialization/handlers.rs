use super::{models::TestRequest, service};
use crate::app::{authentication::AuthToken, device_url};
use axum::{Extension, Json, response::IntoResponse};
use axum_extra::extract::Host;

pub async fn device_initialization_handler(
    Extension(token): Extension<AuthToken>,
    Host(host): Host,
) -> impl IntoResponse {
    let endpoint = device_url(&host, &token.lookup_key);

    Json(service::generate_initialization_response(&endpoint))
}

pub async fn tests_handler(Json(request): Json<TestRequest>) -> impl IntoResponse {
    Json(service::generate_tests_response(&request.test_key))
}
