use std::collections::HashMap;

use super::{models::UpdateStateRequest, service};
use crate::{
    app::{
        authentication::AuthToken,
        error::KoboError,
        state::models::{REVIEWS_MOCK_RESPONSE, StateError},
    },
    client::prosa_client,
};
use axum::{
    Extension, Json,
    extract::{Path, Query},
    response::IntoResponse,
};
use serde_json::Value;

pub async fn get_state_handler(
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let response = service::translate_get_state(prosa_client(), &book_id, &token.api_key).await?;

    Ok(Json(vec![response]))
}

pub async fn update_state_handler(
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<UpdateStateRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let state = request.reading_states.first().ok_or(StateError::MissingState)?;

    let response = service::translate_update_state(prosa_client(), &book_id, state, &token.api_key).await?;

    Ok(Json(response))
}

pub async fn update_rating_handler(
    Extension(token): Extension<AuthToken>,
    Path((book_id, rating)): Path<(String, u8)>,
) -> Result<impl IntoResponse, KoboError> {
    service::translate_update_rating(prosa_client(), &book_id, rating, &token.api_key).await?;

    Ok(())
}

pub async fn get_rating_handler(
    Extension(token): Extension<AuthToken>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let Some(book_id) = params.get("ProductIds") else {
        return Err(StateError::MissingProductId.into());
    };

    let response = service::translate_get_rating(prosa_client(), book_id, &token.api_key).await?;

    Ok(Json(response))
}

pub async fn get_reviews_mock_handler() -> Result<impl IntoResponse, KoboError> {
    let response: Value = serde_json::from_str(REVIEWS_MOCK_RESPONSE).expect("Failed to convert to JSON");

    Ok(Json(response))
}
