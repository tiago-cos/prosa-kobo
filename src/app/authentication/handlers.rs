use crate::{
    CONFIG,
    app::{
        authentication::{models::AuthError, service},
        error::KoboError,
    },
};
use axum::{
    Json,
    extract::{Path, Query},
    response::IntoResponse,
};
use axum_extra::extract::Host;
use std::collections::HashMap;

pub async fn oauth_configs_handler(Host(host): Host, Path(device_id): Path<String>) -> impl IntoResponse {
    let server_url = match &CONFIG.server.public {
        Some(s) => format!("{}://{}:{}", s.scheme, s.host, s.port),
        None if host.contains(':') => format!("http://{host}"),
        _ => format!("http://{host}:{}", CONFIG.server.bind.port),
    };

    Json(service::generate_oauth_config(&server_url, &device_id))
}

pub async fn oauth_token_handler(
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let device_id = params.get("device_id").ok_or(AuthError::MissingDeviceId)?;

    let jwt_token = service::generate_jwt(device_id, CONFIG.auth.token_duration);

    let response = Json(service::generate_oauth_token(
        &jwt_token,
        CONFIG.auth.token_duration,
    ));

    Ok(response)
}
