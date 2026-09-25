use super::service;
use crate::{
    app::{authentication::AuthToken, device_url, error::KoboError},
    client::prosa_client,
};
use axum::{
    Extension, Json,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use axum_extra::extract::Host;

pub async fn device_sync_handler(
    Host(host): Host,
    headers: HeaderMap,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let endpoint = device_url(&host, &token.lookup_key);

    let sync_token = headers
        .get("X-Kobo-Synctoken")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    let (new_sync_token, response) = service::translate_sync(
        prosa_client(),
        sync_token,
        &endpoint,
        &token.api_key,
        &token.device_id,
    )
    .await?;

    let mut headers = HeaderMap::new();
    let sync_header =
        HeaderValue::from_str(&new_sync_token.to_string()).expect("Failed to create sync header");
    headers.insert("X-Kobo-Synctoken", sync_header);

    Ok((headers, Json(response)))
}
