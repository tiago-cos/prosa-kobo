use super::{
    models::{CheckContentRequest, PatchAnnotationsRequest},
    service,
};
use crate::{
    app::{authentication::AuthToken, error::KoboError},
    client::prosa_client,
};
use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};

pub async fn check_for_changes_handler(
    Json(request): Json<Vec<CheckContentRequest>>,
) -> Result<impl IntoResponse, KoboError> {
    let changed = service::get_changed_annotations(request).await?;

    Ok(Json(changed))
}

pub async fn get_annotations_handler(
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    let annotations = service::get_annotations(prosa_client(), &book_id, &token.api_key).await?;
    let etag = service::get_etag(&book_id).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "ETag",
        HeaderValue::from_str(&etag).expect("Failed to create header"),
    );

    let response = (headers, Json(annotations)).into_response();
    Ok(response)
}

pub async fn patch_annotations_handler(
    Path(book_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<PatchAnnotationsRequest>,
) -> Result<impl IntoResponse, KoboError> {
    service::patch_annotations(prosa_client(), &book_id, request, &token.api_key).await?;
    Ok(StatusCode::NO_CONTENT)
}
