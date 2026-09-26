use super::service;
use crate::{
    app::{authentication::AuthToken, error::KoboError},
    client::prosa_client,
};
use axum::{Extension, body::Bytes, extract::Path, http::StatusCode, response::IntoResponse};

pub async fn download_book_handler(
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let book = service::download_book(prosa_client(), &book_id, &token.api_key).await?;

    Ok(Bytes::from_owner(book))
}

pub async fn delete_book_handler(
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    service::delete_book(prosa_client(), &book_id, &token.api_key).await?;

    Ok(StatusCode::NO_CONTENT)
}
