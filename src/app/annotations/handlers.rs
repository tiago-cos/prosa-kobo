use super::{
    models::{CheckContentRequest, PatchAnnotationsRequest},
    service,
};
use crate::app::{authentication::AuthToken, error::KoboError, AppState, Pool, ProsaClient};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    Extension, Json,
};

pub async fn check_for_changes_handler(
    State(pool): State<Pool>,
    Json(request): Json<Vec<CheckContentRequest>>,
) -> Result<impl IntoResponse, KoboError> {
    //TODO remove
    println!("CHECKING FOR CHANGES {:#?}", request);
    let changed = service::filter_changed(&pool, request).await;

    Ok(Json(changed))
}

pub async fn get_annotations_handler(
    State(state): State<AppState>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> impl IntoResponse {
    //TODO remove
    println!("GETTING ANNOTATIONS: {}", book_id);
    let annotations = service::get_annotations(&state.prosa_client, &book_id, &token.api_key).await;
    let etag = service::get_etag(&state.pool, &book_id).await;

    let mut headers = HeaderMap::new();
    headers.insert(
        "ETag",
        HeaderValue::from_str(&etag).expect("Failed to create header"),
    );

    (headers, Json(annotations)).into_response()
}

pub async fn patch_annotations_handler(
    State(client): State<ProsaClient>,
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<PatchAnnotationsRequest>,
) -> impl IntoResponse {
    //TODO remove
    println!("POSTING ANNOTATIONS: {}", book_id);
    service::patch_annotations(&client, &book_id, request, &token.api_key).await
}
