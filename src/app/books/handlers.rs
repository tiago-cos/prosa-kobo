use super::service;
use crate::app::{authentication::AuthToken, error::KoboError, tokens::TokenError, AppState, ProsaClient};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
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

pub async fn delete_book_handler(
    State(client): State<ProsaClient>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    service::delete_book(&client, &book_id, &token.api_key).await;

    Ok(StatusCode::NO_CONTENT)
}
