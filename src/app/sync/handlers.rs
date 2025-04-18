use super::service;
use crate::app::{authentication::AuthToken, error::KoboError, AppState};
use axum::{extract::State, response::IntoResponse, Extension, Json};

pub async fn device_sync_handler(
    State(state): State<AppState>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    //TODO remove
    println!("TRIGGERED SYNC");

    let server_url = format!(
        "http://{}:{}",
        state.config.server.announced_host, state.config.server.announced_port
    );

    let response = service::translate_sync(
        &state.pool,
        &state.prosa_client,
        &server_url,
        state.config.token.expiration,
        &token.api_key,
    )
    .await;

    Ok(Json(response))
}
