use super::service;
use crate::{
    app::{authentication::AuthToken, error::KoboError, server_url},
    client::prosa_client,
};
use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::Host;

pub async fn metadata_handler(
    Host(host): Host,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let endpoint = format!("{}/{}", server_url(&host), token.lookup_key);

    let response = service::translate_metadata(
        prosa_client(),
        &book_id,
        &endpoint,
        &token.api_key,
        &token.device_id,
    )
    .await?;

    Ok(Json(vec![response]))
}
