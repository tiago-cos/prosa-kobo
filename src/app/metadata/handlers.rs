use super::service;
use crate::app::{AppState, authentication::AuthToken, error::KoboError, server_url};
use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use axum_extra::extract::Host;

pub async fn metadata_handler(
    State(state): State<AppState>,
    Host(host): Host,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let endpoint = format!("{}/{}", server_url(&host), token.lookup_key);

    let response = service::translate_metadata(
        &state.pool,
        &state.kepubs,
        &state.prosa_client,
        &book_id,
        &endpoint,
        &token.api_key,
        &token.device_id,
    )
    .await?;

    Ok(Json(vec![response]))
}
