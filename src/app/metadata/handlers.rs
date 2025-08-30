use super::service;
use crate::app::{AppState, authentication::AuthToken, error::KoboError};
use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use axum_extra::extract::Host;

pub async fn metadata_handler(
    State(state): State<AppState>,
    Host(host): Host,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let server_url = match &state.config.server.public {
        Some(s) => format!("{}://{}:{}", s.scheme, s.host, s.port),
        None if host.contains(':') => format!("http://{host}"),
        _ => format!("http://{host}:{}", state.config.server.bind.port),
    };

    let response = service::translate_metadata(
        &state.pool,
        &state.prosa_client,
        &book_id,
        &server_url,
        state.config.download_token.book_expiration,
        &token.api_key,
        &token.device_id,
    )
    .await?;

    Ok(Json(vec![response]))
}
