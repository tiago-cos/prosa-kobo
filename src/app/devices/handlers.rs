use super::{
    models::{
        DeviceAuthRequest, DeviceAuthResponse, DeviceError, LinkDeviceRequest, RefreshTokenRequest,
        RefreshTokenResponse,
    },
    service,
};
use crate::app::{authentication, error::KoboError, AppState, Pool};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub async fn device_auth_handler(
    State(state): State<AppState>,
    Json(body): Json<DeviceAuthRequest>,
) -> impl IntoResponse {
    let device_id = service::generate_device_id(&body.device_id, &body.user_key).await;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;

    service::add_unlinked_device(&state.pool, &device_id, now).await;

    let secret = &state.config.auth.secret_key;
    let token_duration = &state.config.auth.token_duration;
    let refresh_token_duration = &state.config.auth.refresh_token_duration;

    let regular_token = authentication::generate_jwt(secret, &device_id, token_duration).await;
    let refresh_token = authentication::generate_jwt(secret, &device_id, refresh_token_duration).await;

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
    let secret = &state.config.auth.secret_key;
    let token_duration = &state.config.auth.token_duration;
    let refresh_token_duration = &state.config.auth.refresh_token_duration;
    let device_id = authentication::verify_jwt(&body.refresh_token, secret).await?;

    let regular_token = authentication::generate_jwt(secret, &device_id, token_duration).await;
    let refresh_token = authentication::generate_jwt(secret, &device_id, refresh_token_duration).await;

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
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let api_key = match params.get("api_key") {
        Some(key) => key,
        None => return Err(DeviceError::MissingApiKey.into()),
    };

    let device_list = service::get_linked_devices(&pool, &api_key).await?;
    Ok(Json(device_list))
}

pub async fn unlink_device_handler(
    State(pool): State<Pool>,
    Json(body): Json<LinkDeviceRequest>,
) -> Result<(), KoboError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;

    service::unlink_device(&pool, &body.device_id, &body.api_key, now).await?;
    Ok(())
}
