use super::service;
use crate::app::{authentication::AuthToken, error::KoboError, AppState};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};

pub async fn metadata_handler(
    State(state): State<AppState>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let server_url = format!(
        "http://{}:{}",
        state.config.server.announced_host, state.config.server.announced_port
    );

    let response = service::translate_metadata(
        &state.pool,
        &state.prosa_client,
        &book_id,
        &server_url,
        state.config.download_token.book_expiration,
        &token.api_key,
    )
    .await?;

    Ok(Json(vec![response]))
}
