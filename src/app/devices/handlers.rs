use super::{
    models::{
        DeviceAuthRequest, DeviceAuthResponse, LinkDeviceRequest, RefreshTokenRequest, RefreshTokenResponse,
    },
    service,
};
use crate::app::{AppState, Pool, authentication, devices::models::DeviceError, error::KoboError};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
};

pub async fn device_auth_handler(
    State(state): State<AppState>,
    Json(body): Json<DeviceAuthRequest>,
) -> impl IntoResponse {
    let device_id = service::generate_device_id(&body.device_id, &body.user_key);

    let linked_device = service::get_linked_device(&state.pool, &device_id).await;
    let unlinked_device = service::get_unlinked_device(&state.pool, &device_id).await;
    if linked_device.is_none() && unlinked_device.is_none() {
        service::add_unlinked_device(&state.pool, &device_id).await;
    }

    let jwt_key_path = &state.config.auth.jwt_key_path;
    let token_duration = state.config.auth.token_duration;
    let refresh_token_duration = state.config.auth.refresh_token_duration;

    let regular_token = authentication::generate_jwt(jwt_key_path, &device_id, token_duration).await;
    let refresh_token = authentication::generate_jwt(jwt_key_path, &device_id, refresh_token_duration).await;

    Json(DeviceAuthResponse::new(
        &regular_token,
        &refresh_token,
        &body.user_key,
    ))
}

pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let jwt_key_path = &state.config.auth.jwt_key_path;
    let token_duration = state.config.auth.token_duration;
    let refresh_token_duration = state.config.auth.refresh_token_duration;
    let device_id = authentication::verify_jwt(&body.refresh_token, jwt_key_path).await?;

    let regular_token = authentication::generate_jwt(jwt_key_path, &device_id, token_duration).await;
    let refresh_token = authentication::generate_jwt(jwt_key_path, &device_id, refresh_token_duration).await;

    Ok(Json(RefreshTokenResponse::new(&regular_token, &refresh_token)))
}

pub async fn get_unlinked_devices_handler(State(pool): State<Pool>) -> impl IntoResponse {
    Json(service::get_unlinked_devices(&pool).await)
}

pub async fn link_device_handler(
    State(pool): State<Pool>,
    Json(body): Json<LinkDeviceRequest>,
) -> Result<(), KoboError> {
    service::link_device(&pool, &body.device_id, &body.api_key).await?;
    Ok(())
}

pub async fn get_linked_devices_handler(
    State(pool): State<Pool>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, KoboError> {
    let api_key = headers
        .get("api-key")
        .and_then(|value| value.to_str().ok())
        .ok_or(DeviceError::MissingApiKey)?;

    let device_list = service::get_linked_devices(&pool, api_key).await?;
    Ok(Json(device_list))
}

pub async fn unlink_device_handler(
    State(pool): State<Pool>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> Result<(), KoboError> {
    let api_key = headers
        .get("api-key")
        .and_then(|value| value.to_str().ok())
        .ok_or(DeviceError::MissingApiKey)?;

    service::unlink_device(&pool, &device_id, api_key).await?;
    Ok(())
}
