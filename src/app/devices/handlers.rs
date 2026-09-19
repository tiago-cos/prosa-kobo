use super::{
    models::{LinkDeviceRequest, LinkDeviceResponse, LinkedDeviceResponse, ListLinkedDevicesQuery},
    service,
};
use crate::{
    CONFIG,
    app::{AppState, Pool, authentication::ProsaToken, error::KoboError},
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::Host;

pub async fn link_device_handler(
    State(state): State<AppState>,
    Extension(token): Extension<ProsaToken>,
    Host(host): Host,
    Json(body): Json<LinkDeviceRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let (device_id, lookup_key) = service::link_device(
        &state.pool,
        state.prosa_client.as_ref(),
        token.owner_or_self(body.user_id.as_deref()),
        &body.name,
        &body.api_key,
    )
    .await?;

    let server_url = match &CONFIG.server.public {
        Some(s) => format!("{}://{}:{}", s.scheme, s.host, s.port),
        None if host.contains(':') => format!("http://{host}"),
        _ => format!("http://{host}:{}", CONFIG.server.bind.port),
    };

    Ok(Json(LinkDeviceResponse {
        device_id,
        api_endpoint: format!("{server_url}/{lookup_key}"),
    }))
}

pub async fn get_linked_devices_handler(
    State(pool): State<Pool>,
    Query(query): Query<ListLinkedDevicesQuery>,
) -> impl IntoResponse {
    let devices: Vec<LinkedDeviceResponse> = service::get_linked_devices(&pool, query.user_id.as_deref())
        .await
        .into_iter()
        .map(Into::into)
        .collect();

    Json(devices)
}

pub async fn unlink_device_handler(
    State(pool): State<Pool>,
    Path(device_id): Path<String>,
) -> Result<StatusCode, KoboError> {
    service::unlink_device(&pool, &device_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
