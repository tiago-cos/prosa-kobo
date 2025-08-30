use super::{models::TestRequest, service};
use crate::app::{AppState, authentication::AuthToken};
use axum::{Extension, Json, extract::State, response::IntoResponse};
use axum_extra::extract::Host;

pub async fn device_initialization_handler(
    State(state): State<AppState>,
    Host(host): Host,
    Extension(token): Extension<AuthToken>,
) -> impl IntoResponse {
    let server_url = match &state.config.server.public {
        Some(s) => format!("{}://{}:{}", s.scheme, s.host, s.port),
        None if host.contains(':') => format!("http://{host}"),
        _ => format!("http://{host}:{}", state.config.server.bind.port),
    };

    Json(service::generate_initialization_response(
        &server_url,
        &token.device_id,
    ))
}

pub async fn tests_handler(Json(request): Json<TestRequest>) -> impl IntoResponse {
    Json(service::generate_tests_response(&request.test_key))
}
