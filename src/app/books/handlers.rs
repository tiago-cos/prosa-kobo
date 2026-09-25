use super::service;
use crate::{
    app::{annotations, authentication::AuthToken, error::KoboError, kepub},
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
    service::delete_book(prosa_client(), &book_id, &token.api_key)?;
    annotations::service::delete_etag(&book_id).await?;
    kepub::evict(&book_id);

    Ok(StatusCode::NO_CONTENT)
}
