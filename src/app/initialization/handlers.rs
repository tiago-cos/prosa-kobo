use super::{models::TestRequest, service};
use crate::app::{authentication::AuthToken, initialization::service::generate_tests_response, Config};
use axum::{extract::State, response::IntoResponse, Extension, Json};

pub async fn device_initialization_handler(
    State(config): State<Config>,
    Extension(token): Extension<AuthToken>,
) -> impl IntoResponse {
    //TODO remove
    println!("INITIALIZATION");
    let host = format!(
        "{}:{}",
        &config.server.announced_host, &config.server.announced_port
    );
    Json(service::generate_initialization_response(&host, &token.device_id).await)
}

pub async fn tests_handler(Json(request): Json<TestRequest>) -> impl IntoResponse {
    generate_tests_response(&request.test_key).await
}
