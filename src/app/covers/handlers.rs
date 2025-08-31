use super::service;
use crate::app::{AppState, covers::models::CoverTokenError, error::KoboError};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use log::warn;
use std::collections::HashMap;

pub async fn download_cover_handler(
    State(state): State<AppState>,
    Path(book_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let Some(cover_token) = params.get("token") else {
        return Err(CoverTokenError::InvalidToken.into());
    };

    let mut cover = service::download_cover(&state.pool, &state.prosa_client, &book_id, cover_token).await?;

    let width: Option<u32> = params.get("width").and_then(|s| s.parse().ok());
    let height: Option<u32> = params.get("height").and_then(|s| s.parse().ok());

    if let (Some(w), Some(h)) = (width, height) {
        match service::resize_cover(&cover, w, h) {
            Ok(c) => cover = c,
            _ => warn!("Failed to resize image."),
        }
    }

    Ok(cover)
}
