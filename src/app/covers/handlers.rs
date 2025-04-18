use super::service;
use crate::app::{authentication::AuthError, error::KoboError, AppState};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use std::collections::HashMap;

pub async fn download_cover_handler(
    State(state): State<AppState>,
    Path(book_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let device_id = match params.get("device_id") {
        Some(id) => id,
        None => return Err(AuthError::MissingAuth.into()),
    };

    let cover = service::download_cover(&state.pool, &state.prosa_client, &book_id, &device_id).await?;
    Ok(cover)
}
