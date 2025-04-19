use super::{models::UpdateStateRequest, service};
use crate::app::{authentication::AuthToken, error::KoboError, ProsaClient};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};

pub async fn get_state_handler(
    State(client): State<ProsaClient>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let response = service::translate_get_state(&client, &book_id, &token.api_key).await;

    Ok(Json(vec![response]))
}

//TODO can't assume ill never have errors, api keys can have different capabilities
pub async fn update_state_handler(
    State(client): State<ProsaClient>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<UpdateStateRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let state = request.reading_states.first().expect("State should be present");
    let response = service::translate_update_state(&client, &book_id, state, &token.api_key).await;

    Ok(Json(response))
}
