use super::{
    models::{LinkDeviceRequest, LinkDeviceResponse, LinkedDeviceResponse, ListLinkedDevicesQuery},
    service,
};
use crate::{
    CONFIG,
    app::{authentication::ProsaToken, error::KoboError},
    client::prosa_client,
};
use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::Host;

pub async fn link_device_handler(
    Extension(token): Extension<ProsaToken>,
    Host(host): Host,
    Json(body): Json<LinkDeviceRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let (device_id, lookup_key) = service::link_device(
        prosa_client(),
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
    Query(query): Query<ListLinkedDevicesQuery>,
) -> Result<impl IntoResponse, KoboError> {
    let devices: Vec<LinkedDeviceResponse> = service::get_linked_devices(query.user_id.as_deref())
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(devices))
}

pub async fn unlink_device_handler(Path(device_id): Path<String>) -> Result<StatusCode, KoboError> {
    service::unlink_device(&device_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
