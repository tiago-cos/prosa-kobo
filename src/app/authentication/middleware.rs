use super::{models::AuthError, service};
use crate::app::{
    authentication::models::{AuthToken, ProsaToken},
    devices,
    error::KoboError,
    tracing::LoggedPath,
};
use axum::{
    extract::Request,
    http::{self, HeaderMap, HeaderValue, Uri, uri::PathAndQuery},
    middleware::Next,
    response::{IntoResponse, Response},
};
use log::warn;

const MAX_CLIENT_DEVICE_ID: usize = 128;
const UNKEYED_PREFIX: &str = "/api/v3/content/";

/// The lookup key reaches the middleware as the first path segment, stripped
/// here so routes see the paths they were written for.
///
/// Requests the device builds from `reading_services_host` are the exception.
/// That setting is a bare origin rather than a URL the device appends to, so
/// the device keeps only its scheme, host and port and everything under
/// `/api/v3/content` arrives with no key. Those are matched on the hardware id
/// the device sends instead, which keyed requests record as they pass.
///
/// Every response is marked with the path to log, which leaves out the key
/// whether or not it named a device, so a key never reaches the log.
pub async fn extract_device_middleware(request: Request, next: Next) -> Response {
    let logged = LoggedPath(without_lookup_key(request.uri().path()));

    let mut response = match identify_device(request, next).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    };
    response.extensions_mut().insert(logged);

    response
}

fn without_lookup_key(path: &str) -> String {
    if path.starts_with(UNKEYED_PREFIX) {
        return path.to_owned();
    }

    let remainder = path
        .trim_start_matches('/')
        .split_once('/')
        .map_or("", |(_, rest)| rest);

    format!("/{remainder}")
}

async fn identify_device(mut request: Request, next: Next) -> Result<Response, KoboError> {
    let path = request.uri().path().to_owned();
    let mut segments = path.trim_start_matches('/').splitn(2, '/');

    let lookup_key = segments.next().unwrap_or_default();
    let remainder = segments.next().unwrap_or_default();
    let client_device_id = client_device_id(request.headers()).map(str::to_owned);

    let device = if let Some(device) = devices::service::get_device_by_lookup_key(lookup_key).await? {
        *request.uri_mut() = strip_lookup_key(request.uri(), remainder)?;

        if let Some(client_device_id) = &client_device_id
            && device.client_device_id.as_ref() != Some(client_device_id)
        {
            devices::service::claim_client_device_id(&device.device_id, client_device_id).await?;
        }

        device
    } else {
        let Some(client_device_id) = &client_device_id else {
            report_unrecognized(&request);
            return Err(AuthError::UnauthenticatedDevice.into());
        };

        devices::service::get_device_by_client_id(client_device_id)
            .await?
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
        without_lookup_key(request.uri().path()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_the_lookup_key_out_of_a_path() {
        assert_eq!(
            without_lookup_key("/-w2Sk82Q_jFWuL79rcgiDhfiSxJ5XIIrgRdabAaCfts/v1/library/sync"),
            "/v1/library/sync"
        );
        assert_eq!(
            without_lookup_key("/-w2Sk82Q_jFWuL79rcgiDhfiSxJ5XIIrgRdabAaCfts"),
            "/"
        );
    }

    #[test]
    fn keeps_a_path_that_carries_no_key() {
        assert_eq!(
            without_lookup_key("/api/v3/content/checkforchanges"),
            "/api/v3/content/checkforchanges"
        );
    }
}
