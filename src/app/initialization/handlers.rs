use super::{models::TestRequest, service};
use crate::{CONFIG, app::authentication::AuthToken};
use axum::{Extension, Json, response::IntoResponse};
use axum_extra::extract::Host;

pub async fn device_initialization_handler(
    Host(host): Host,
    Extension(token): Extension<AuthToken>,
) -> impl IntoResponse {
    let server_url = match &CONFIG.server.public {
        Some(s) => format!("{}://{}:{}", s.scheme, s.host, s.port),
        None if host.contains(':') => format!("http://{host}"),
        _ => format!("http://{host}:{}", CONFIG.server.bind.port),
    };

    Json(service::generate_initialization_response(
        &server_url,
        &token.device_id,
    ))
}

pub async fn tests_handler(Json(request): Json<TestRequest>) -> impl IntoResponse {
    Json(service::generate_tests_response(&request.test_key))
}
