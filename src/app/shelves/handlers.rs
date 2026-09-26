use crate::{
    app::{
        authentication::AuthToken,
        error::KoboError,
        shelves::{
            models::{
                AddBooksToShelfRequest, CreateShelfRequest, DeleteBooksFromShelfRequest, RenameShelfRequest,
            },
            service,
        },
    },
    client::prosa_client,
};
use axum::{Extension, Json, extract::Path, http::StatusCode, response::IntoResponse};

pub async fn create_shelf_handler(
    Extension(token): Extension<AuthToken>,
    Json(request): Json<CreateShelfRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let shelf_id = service::translate_create_shelf(prosa_client(), &request, &token.api_key).await?;

    Ok((StatusCode::CREATED, shelf_id))
}

pub async fn delete_shelf_handler(
    Path(shelf_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    service::translate_delete_shelf(prosa_client(), &shelf_id, &token.api_key).await?;

    Ok(())
}

pub async fn rename_shelf_handler(
    Path(shelf_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<RenameShelfRequest>,
) -> Result<impl IntoResponse, KoboError> {
    service::translate_rename_shelf(prosa_client(), &shelf_id, &request.name, &token.api_key).await?;

    Ok(())
}

pub async fn add_book_to_shelf_handler(
    Path(shelf_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<AddBooksToShelfRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let added =
        service::translate_add_books_to_shelf(prosa_client(), &shelf_id, &request.items, &token.api_key)
            .await?;

    Ok((StatusCode::CREATED, Json(added)))
}

pub async fn delete_books_from_shelf_handler(
    Path(shelf_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<DeleteBooksFromShelfRequest>,
) -> Result<impl IntoResponse, KoboError> {
    service::translate_delete_books_from_shelf(prosa_client(), &shelf_id, &request.items, &token.api_key)
        .await?;

    Ok(())
}
