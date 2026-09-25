use super::service;
use crate::{
    app::{authentication::AuthToken, error::KoboError},
    client::prosa_client,
};
use axum::{
    Extension,
    extract::{Path, Query},
    response::IntoResponse,
};
use log::warn;
use std::collections::HashMap;

pub async fn download_cover_handler(
    Path(book_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let mut cover = service::download_cover(prosa_client(), &book_id, &token.api_key)?;

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
