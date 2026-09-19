use super::{
    models::{
        DeviceAuthRequest, DeviceAuthResponse, LinkDeviceRequest, ListLinkedDevicesQuery,
        RefreshTokenRequest, RefreshTokenResponse,
    },
    service,
};
use crate::{
    CONFIG,
    app::{AppState, Pool, authentication, authentication::ProsaToken, error::KoboError},
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
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

    let regular_token = authentication::generate_jwt(&device_id, CONFIG.auth.token_duration);
    let refresh_token = authentication::generate_jwt(&device_id, CONFIG.auth.refresh_token_duration);

    Json(DeviceAuthResponse::new(
        &regular_token,
        &refresh_token,
        &body.user_key,
    ))
}

pub async fn refresh_token_handler(
    Json(body): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let device_id = authentication::verify_jwt(&body.refresh_token)?;

    let regular_token = authentication::generate_jwt(&device_id, CONFIG.auth.token_duration);
    let refresh_token = authentication::generate_jwt(&device_id, CONFIG.auth.refresh_token_duration);

    Ok(Json(RefreshTokenResponse::new(&regular_token, &refresh_token)))
}

pub async fn get_unlinked_devices_handler(State(pool): State<Pool>) -> impl IntoResponse {
    Json(service::get_unlinked_devices(&pool).await)
}

pub async fn link_device_handler(
    State(state): State<AppState>,
    Extension(token): Extension<ProsaToken>,
    Json(body): Json<LinkDeviceRequest>,
) -> Result<StatusCode, KoboError> {
    service::link_device(
        &state.pool,
        state.prosa_client.as_ref(),
        &body.device_id,
        token.owner_or_self(body.user_id.as_deref()),
        &body.api_key,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_linked_devices_handler(
    State(pool): State<Pool>,
    Query(query): Query<ListLinkedDevicesQuery>,
) -> impl IntoResponse {
    Json(service::get_linked_devices(&pool, query.user_id.as_deref()).await)
}

pub async fn unlink_device_handler(
    State(pool): State<Pool>,
    Path(device_id): Path<String>,
) -> Result<StatusCode, KoboError> {
    service::unlink_device(&pool, &device_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
