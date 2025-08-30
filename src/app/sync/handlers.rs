use super::service;
use crate::app::{AppState, authentication::AuthToken, error::KoboError};
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use axum_extra::extract::Host;

pub async fn device_sync_handler(
    State(state): State<AppState>,
    Host(host): Host,
    headers: HeaderMap,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let server_url = match &state.config.server.public {
        Some(s) => format!("{}://{}:{}", s.scheme, s.host, s.port),
        None if host.contains(':') => format!("http://{host}"),
        _ => format!("http://{host}:{}", state.config.server.bind.port),
    };

    let since = headers
        .get("X-Kobo-Synctoken")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    let sync_token = service::create_new_sync_token();
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
