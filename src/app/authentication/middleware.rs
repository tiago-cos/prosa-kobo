use super::{models::AuthError, service};
use crate::app::{
    Pool,
    authentication::models::{AuthToken, ProsaToken},
    devices,
    error::KoboError,
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Uri, uri::PathAndQuery},
    middleware::Next,
    response::IntoResponse,
};

pub async fn extract_device_middleware(
    State(pool): State<Pool>,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, KoboError> {
    let path = request.uri().path().to_owned();
    let mut segments = path.trim_start_matches('/').splitn(2, '/');

    let lookup_key = segments.next().unwrap_or_default();
    let remainder = segments.next().unwrap_or_default();

    let device = devices::service::get_device_by_lookup_key(&pool, lookup_key)
        .await
        .ok_or(AuthError::UnauthenticatedDevice)?;

    *request.uri_mut() = strip_lookup_key(request.uri(), remainder)?;

    request.extensions_mut().insert(AuthToken {
        device_id: device.device_id,
        lookup_key: device.lookup_key,
        api_key: device.api_key,
    });

    Ok(next.run(request).await)
}

fn strip_lookup_key(uri: &Uri, remainder: &str) -> Result<Uri, AuthError> {
    let query = uri.query().map(|query| format!("?{query}")).unwrap_or_default();

    let path_and_query = format!("/{remainder}{query}")
        .parse::<PathAndQuery>()
        .or(Err(AuthError::InternalError))?;

    let mut parts = uri.clone().into_parts();
    parts.path_and_query = Some(path_and_query);

    Uri::from_parts(parts).or(Err(AuthError::InternalError))
}

pub async fn extract_prosa_token_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, KoboError> {
    let header = headers.get("Authorization").ok_or(AuthError::MissingAuth)?;
    let token: ProsaToken = service::verify_prosa_jwt(bearer_token(header)?)?;

    request.extensions_mut().insert(token);

    Ok(next.run(request).await)
}

fn bearer_token(header: &HeaderValue) -> Result<&str, AuthError> {
    let header = header.to_str().or(Err(AuthError::InvalidAuthHeader))?;

    header
        .split_whitespace()
        .nth(1)
        .ok_or(AuthError::InvalidAuthHeader)
}
