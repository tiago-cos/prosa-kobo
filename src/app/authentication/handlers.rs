use super::{
    models::{AuthToken, DEVICE_AUTH_RESPONSE, OAUTH_TOKEN},
    service,
};
use crate::app::{Host, device_url};
use axum::{Extension, Json, response::IntoResponse};

pub async fn oauth_configs_handler(
    Extension(token): Extension<AuthToken>,
    Host(host): Host,
) -> impl IntoResponse {
    let endpoint = device_url(&host, &token.lookup_key);

    Json(service::generate_oauth_config(&endpoint))
}

pub async fn device_auth_handler() -> impl IntoResponse {
    Json(DEVICE_AUTH_RESPONSE)
}

pub async fn oauth_token_handler() -> impl IntoResponse {
    Json(OAUTH_TOKEN)
}
