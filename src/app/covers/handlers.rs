use super::{models::CoverSize, service};
use crate::{
    app::{authentication::AuthToken, error::KoboError},
    client::prosa_client,
};
use axum::{
    Extension,
    extract::{Path, Query},
    response::IntoResponse,
};

pub async fn download_cover_handler(
    Path(book_id): Path<String>,
    Query(size): Query<CoverSize>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let cover = service::translate_cover(prosa_client(), &book_id, &size, &token.api_key).await?;

    Ok(cover)
}
