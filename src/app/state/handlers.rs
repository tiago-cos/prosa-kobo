use std::collections::HashMap;

use super::{models::UpdateStateRequest, service};
use crate::app::{
    authentication::AuthToken,
    error::KoboError,
    state::models::{StateError, REVIEWS_MOCK_RESPONSE},
    ProsaClient,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension, Json,
};
use serde_json::Value;

pub async fn get_state_handler(
    State(client): State<ProsaClient>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let response = service::translate_get_state(&client, &book_id, &token.api_key).await?;

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
    let response = service::translate_update_state(&client, &book_id, state, &token.api_key).await?;

    Ok(Json(response))
}

//TODO here's how annotations work: the kobo devices stores an ETag alongisde each bookId. Before requesting annotations, it sends a request to /api/v3/content/checkforchanges
// with the bookIds and the etag of each book. The response to this request are the bookIds of the books whose annotations have changed (have a different etag than expected).
// After this, the kobo makes a request to /api/v3/content/{book_id}/annotations. This endpoint returns the book annotations and a new etag. The kobo then stores this etag
// alongside the requested bookId, ending the cycle.
// Here's how I'm going to implement this: In prosa, we do it like normal, where each book has an "annotations last changed" timestamp, and if the timestamp is higher
// than the /sync since query parameter, we return that book_id. Can't have a "since" parameter since that would entail recording deletes as well, but the kobo treats the
// response body as _all_ the annotations, meaning I'd need to keep track of all annotations here anyways. Might as well just send everything.
// Then, in this middleware, I'll have an annotations table, where I store a relationship between a book_id, its etag. Then, in the /sync endpoint, we iterate over the
// annotation changed book ids, and we go and change their etag here in the relationship.
// by doing this, when the kobo next checks for changes, the book ids of the books that were affected will be returned. After that, the kobo will get the annotations of
// each book that has had annotations changed and I'll return the changed etag in the headers.

//New things: In the annotation requests from kobo -> server, the kobo only includes added annotations since the last 204 response from the server annotations endpoint.
//If an annotation was deleted between the last 204 response, the kobo will send the annotation id in a deleted array.

pub async fn update_rating_handler(
    State(client): State<ProsaClient>,
    Extension(token): Extension<AuthToken>,
    Path((book_id, rating)): Path<(String, u8)>,
) -> Result<impl IntoResponse, KoboError> {
    service::translate_update_rating(&client, &book_id, rating, &token.api_key).await?;

    Ok(())
}

pub async fn get_rating_handler(
    State(client): State<ProsaClient>,
    Extension(token): Extension<AuthToken>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, KoboError> {
    let book_id = match params.get("ProductIds") {
        Some(id) => id,
        None => return Err(StateError::MissingProductId.into()),
    };

    let response = service::translate_get_rating(&client, book_id, &token.api_key).await?;

    Ok(Json(response))
}

pub async fn get_reviews_mock_handler() -> Result<impl IntoResponse, KoboError> {
    let response: Value = serde_json::from_str(&REVIEWS_MOCK_RESPONSE).expect("Failed to convert to JSON");

    Ok(Json(response))
}
