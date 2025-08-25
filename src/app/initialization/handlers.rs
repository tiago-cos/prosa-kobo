use super::{models::TestRequest, service};
use crate::app::{Config, authentication::AuthToken};
use axum::{Extension, Json, extract::State, response::IntoResponse};

pub async fn device_initialization_handler(
    State(config): State<Config>,
    Extension(token): Extension<AuthToken>,
) -> impl IntoResponse {
    let host = format!(
        "{}:{}",
        &config.server.announced_host, &config.server.announced_port
    );
    Json(service::generate_initialization_response(&host, &token.device_id))
}

pub async fn tests_handler(Json(request): Json<TestRequest>) -> impl IntoResponse {
    Json(service::generate_tests_response(&request.test_key))
}
