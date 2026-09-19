use crate::app::{
    Pool,
    authentication::{
        ProsaToken,
        models::{AuthError, CREATE, DELETE, READ},
    },
    devices::{self, models::LinkDeviceRequest},
    error::KoboError,
};
use axum::{
    Extension, Json,
    body::{Body, to_bytes},
    extract::{FromRequest, Path, Query, Request, State},
    middleware::Next,
    response::IntoResponse,
};
use std::collections::HashMap;

pub async fn can_read_unlinked_devices(
    Extension(token): Extension<ProsaToken>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, KoboError> {
    if !token.can(READ) {
        return Err(AuthError::Forbidden.into());
    }

    Ok(next.run(request).await)
}

pub async fn can_search_linked_devices(
    Extension(token): Extension<ProsaToken>,
    Query(params): Query<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, KoboError> {
    if !token.can(READ) {
        return Err(AuthError::Forbidden.into());
    }

    if token.is_admin {
        return Ok(next.run(request).await);
    }

    match params.get("user_id") {
        Some(user_id) if token.can_act_for(user_id) => Ok(next.run(request).await),
        _ => Err(AuthError::Forbidden.into()),
    }
}

pub async fn can_link_device(
    Extension(token): Extension<ProsaToken>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, KoboError> {
    if !token.can(CREATE) {
        return Err(AuthError::Forbidden.into());
    }

    let (parts, body) = request.into_parts();
    let body = to_bytes(body, usize::MAX)
        .await
        .map_err(|_| AuthError::InvalidAuthHeader)?;

    let inspected = Request::from_parts(parts.clone(), Body::from(body.clone()));
    let forwarded = Request::from_parts(parts, Body::from(body));

    if let Ok(Json(request)) = Json::<LinkDeviceRequest>::from_request(inspected, &()).await
        && !token.can_act_for(token.owner_or_self(request.user_id.as_deref()))
    {
        return Err(AuthError::Forbidden.into());
    }

    Ok(next.run(forwarded).await)
}

pub async fn can_unlink_device(
    Extension(token): Extension<ProsaToken>,
    State(pool): State<Pool>,
    Path(device_id): Path<String>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, KoboError> {
    if !token.can(DELETE) {
        return Err(AuthError::Forbidden.into());
    }

    if let Some(device) = devices::service::get_linked_device(&pool, &device_id).await
        && !token.can_act_for(&device.user_id)
    {
        return Err(AuthError::Forbidden.into());
    }

    Ok(next.run(request).await)
}
