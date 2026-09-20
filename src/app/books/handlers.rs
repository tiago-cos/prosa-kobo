use super::service;
use crate::app::{AppState, annotations, authentication::AuthToken, error::KoboError, kepub};
use axum::{
    Extension,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn download_book_handler(
    State(state): State<AppState>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let book = service::download_book(&state.kepubs, &state.prosa_client, &book_id, &token.api_key).await?;

    Ok(Bytes::from_owner(book))
}

pub async fn delete_book_handler(
    State(state): State<AppState>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    service::delete_book(state.prosa_client.as_ref(), &book_id, &token.api_key)?;
    annotations::service::delete_etag(&state.pool, &book_id).await;
    kepub::evict(&state.kepubs, &book_id);

    Ok(StatusCode::NO_CONTENT)
}
