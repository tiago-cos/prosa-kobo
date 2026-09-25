use super::{
    models::{AuthToken, DEVICE_AUTH_RESPONSE},
    service,
};
use crate::app::device_url;
use axum::{Extension, Json, response::IntoResponse};
use axum_extra::extract::Host;
use serde_json::Value;

pub async fn oauth_configs_handler(
    Extension(token): Extension<AuthToken>,
    Host(host): Host,
) -> impl IntoResponse {
    let endpoint = device_url(&host, &token.lookup_key);

    Json(service::generate_oauth_config(&endpoint))
}

pub async fn device_auth_handler() -> impl IntoResponse {
    let response: Value = serde_json::from_str(DEVICE_AUTH_RESPONSE).expect("Failed to parse JSON");

    Json(response)
}

pub async fn oauth_token_handler() -> impl IntoResponse {
    Json(service::generate_oauth_token())
}
