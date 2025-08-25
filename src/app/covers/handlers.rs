use super::service;
use crate::app::{AppState, covers::models::CoverTokenError, error::KoboError};
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
    let Some(cover_token) = params.get("token") else {
        return Err(CoverTokenError::InvalidToken.into());
    };

    let cover = service::download_cover(&state.pool, &state.prosa_client, &book_id, cover_token).await?;
    Ok(cover)
}
