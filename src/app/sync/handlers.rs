use super::service;
use crate::app::{AppState, authentication::AuthToken, error::KoboError};
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};

pub async fn device_sync_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let server_url = format!(
        "http://{}:{}",
        state.config.server.announced_host, state.config.server.announced_port
    );

    let since = headers
        .get("X-Kobo-Synctoken")
        .map(|s| s.to_str().ok())
        .flatten()
        .map(|s| s.parse::<i64>().ok())
        .flatten();

    let sync_token = service::create_new_sync_token().await;
    let mut headers = HeaderMap::new();
    let sync_header = HeaderValue::from_str(&sync_token).expect("Failed to create sync header");
    headers.insert("X-Kobo-Synctoken", sync_header);

    let response = service::translate_sync(
        &state.pool,
        &state.prosa_client,
        since,
        &server_url,
        state.config.download_token.book_expiration,
        &token.api_key,
        &token.device_id,
    )
    .await?;

    Ok((headers, Json(response)))
}
