use super::{models::AuthError, service};
use crate::app::{authentication::models::AuthToken, devices, error::KoboError, AppState};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::IntoResponse,
};

pub async fn extract_token_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, KoboError> {
    let jwt_header = headers.get("Authorization");

    let device_id = match jwt_header {
        Some(header) => handle_jwt(&state.config.auth.secret_key, header).await?,
        _ => Err(AuthError::MissingAuth)?,
    };

    let device = match devices::service::get_linked_device(&state.pool, &device_id).await {
        Some(device) => device,
        _ => Err(AuthError::InvalidToken)?,
    };

    request.extensions_mut().insert(AuthToken {
        device_id,
        api_key: device.api_key,
    });
    Ok(next.run(request).await)
}

async fn handle_jwt(secret: &str, header: &HeaderValue) -> Result<String, AuthError> {
    let header = header.to_str().expect("Failed to convert jwt header to string");

    let (_, token) = header
        .split_whitespace()
        .collect::<Vec<_>>()
        .get(0..2)
        .and_then(|parts| Some((parts[0], parts[1])))
        .ok_or(AuthError::InvalidAuthHeader)?;

    let device_id = service::verify_jwt(token, secret).await?;

    Ok(device_id)
}
