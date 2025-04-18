use super::service;
use crate::app::{error::KoboError, tokens::TokenError, AppState};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use std::collections::HashMap;

pub async fn download_book_handler(
    State(state): State<AppState>,
    Path(book_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let book_token = match params.get("token") {
        Some(t) => t,
        None => return Err(TokenError::InvalidToken.into()),
    };

    let book = service::download_book(&state.pool, &state.prosa_client, &book_id, &book_token).await?;
    Ok(book)
}
