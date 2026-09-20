use super::{models::AuthError, service};
use crate::app::{
    Pool,
    authentication::models::{AuthToken, ProsaToken},
    devices,
    error::KoboError,
};
use axum::{
    extract::{Request, State},
    http::{self, HeaderMap, HeaderValue, Uri, uri::PathAndQuery},
    middleware::Next,
    response::IntoResponse,
};
use log::warn;

const MAX_CLIENT_DEVICE_ID: usize = 128;

/// The lookup key reaches the middleware as the first path segment, stripped
/// here so routes see the paths they were written for.
///
/// Requests the device builds from `reading_services_host` are the exception.
/// That setting is a bare origin rather than a URL the device appends to, so
/// the device keeps only its scheme, host and port and everything under
/// `/api/v3/content` arrives with no key. Those are matched on the hardware id
/// the device sends instead, which keyed requests record as they pass.
pub async fn extract_device_middleware(
    State(pool): State<Pool>,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, KoboError> {
    let path = request.uri().path().to_owned();
    let mut segments = path.trim_start_matches('/').splitn(2, '/');

    let lookup_key = segments.next().unwrap_or_default();
    let remainder = segments.next().unwrap_or_default();
    let client_device_id = client_device_id(request.headers()).map(str::to_owned);

    let device = if let Some(device) = devices::service::get_device_by_lookup_key(&pool, lookup_key).await {
        *request.uri_mut() = strip_lookup_key(request.uri(), remainder)?;

        if let Some(client_device_id) = &client_device_id
            && device.client_device_id.as_ref() != Some(client_device_id)
        {
            devices::service::claim_client_device_id(&pool, &device.device_id, client_device_id).await;
        }

        device
    } else {
        let Some(client_device_id) = &client_device_id else {
            report_unrecognized(&request);
            return Err(AuthError::UnauthenticatedDevice.into());
        };

        devices::service::get_device_by_client_id(&pool, client_device_id)
            .await
            .ok_or(AuthError::UnauthenticatedDevice)?
    };

    request.extensions_mut().insert(AuthToken {
        device_id: device.device_id,
        lookup_key: device.lookup_key,
        api_key: device.api_key,
    });

    Ok(next.run(request).await)
}

fn client_device_id(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("x-kobo-deviceid")?.to_str().ok()?.trim();

    (!value.is_empty() && value.len() <= MAX_CLIENT_DEVICE_ID).then_some(value)
}

fn report_unrecognized(request: &Request) {
    let kobo: Vec<String> = request
        .headers()
        .iter()
        .filter(|(name, _)| name.as_str().starts_with("x-kobo"))
        .map(|(name, value)| format!("{name}: {}", value.to_str().unwrap_or("<not text>")))
        .collect();

    let names: Vec<&str> = request.headers().keys().map(http::HeaderName::as_str).collect();

    warn!(
        "{} {} named no device. Headers: [{}]. Kobo headers: [{}]",
        request.method(),
        request.uri().path(),
        names.join(", "),
        kobo.join(", ")
    );
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
