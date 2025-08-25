use std::collections::HashMap;

use super::{models::UpdateStateRequest, service};
use crate::app::{
    ProsaClient,
    authentication::AuthToken,
    error::KoboError,
    state::models::{REVIEWS_MOCK_RESPONSE, StateError},
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde_json::Value;

pub async fn get_state_handler(
    State(client): State<ProsaClient>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let response = service::translate_get_state(&client, &book_id, &token.api_key)?;

    Ok(Json(vec![response]))
}

pub async fn update_state_handler(
    State(client): State<ProsaClient>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<UpdateStateRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let state = request.reading_states.first().ok_or(StateError::MissingState)?;

    let response = service::translate_update_state(&client, &book_id, state, &token.api_key)?;

    Ok(Json(response))
}

pub async fn update_rating_handler(
    State(client): State<ProsaClient>,
    Extension(token): Extension<AuthToken>,
    Path((book_id, rating)): Path<(String, u8)>,
) -> Result<impl IntoResponse, KoboError> {
    service::translate_update_rating(&client, &book_id, rating, &token.api_key)?;

    Ok(())
}

pub async fn get_rating_handler(
    State(client): State<ProsaClient>,
    Extension(token): Extension<AuthToken>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let Some(book_id) = params.get("ProductIds") else {
        return Err(StateError::MissingProductId.into());
    };

    let response = service::translate_get_rating(&client, book_id, &token.api_key)?;

    Ok(Json(response))
}

pub async fn get_reviews_mock_handler() -> Result<impl IntoResponse, KoboError> {
    let response: Value = serde_json::from_str(REVIEWS_MOCK_RESPONSE).expect("Failed to convert to JSON");

    Ok(Json(response))
}
