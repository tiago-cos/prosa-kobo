use crate::app::{authentication::service, Config};
use axum::{
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
    service::generate_oauth_config(&host, &device_id).await
}

pub async fn oauth_token_handler(
    State(config): State<Config>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let device_id = params.get("device_id").expect("device_id should be present");
    let jwt_token =
        service::generate_jwt(&config.auth.secret_key, device_id, &config.auth.token_duration).await;

    service::generate_oauth_token(&jwt_token, config.auth.token_duration).await
}
