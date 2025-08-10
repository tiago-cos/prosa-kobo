use crate::app::{
    Config,
    authentication::{models::AuthError, service},
    error::KoboError,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use std::collections::HashMap;

pub async fn oauth_configs_handler(
    State(config): State<Config>,
    Path(device_id): Path<String>,
) -> impl IntoResponse {
    let host = format!(
        "{}:{}",
        &config.server.announced_host, &config.server.announced_port
    );

    let scheme = &config.server.announced_scheme;

    Json(service::generate_oauth_config(&host, &device_id, scheme).await)
}

pub async fn oauth_token_handler(
    State(config): State<Config>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let device_id = params.get("device_id").ok_or(AuthError::MissingDeviceId)?;

    let jwt_token =
        service::generate_jwt(&config.auth.secret_key, device_id, &config.auth.token_duration).await;

    let response = Json(service::generate_oauth_token(&jwt_token, config.auth.token_duration).await);
    Ok(response)
}
