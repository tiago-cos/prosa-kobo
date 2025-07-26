use crate::app::{
    authentication::AuthToken,
    error::KoboError,
    shelves::{
        models::{
            AddBooksToShelfRequest, CreateShelfRequest, DeleteBooksFromShelfRequest, RenameShelfRequest,
        },
        service,
    },
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

pub async fn create_shelf_handler(
    State(state): State<AppState>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<CreateShelfRequest>,
) -> Result<impl IntoResponse, KoboError> {
    let shelf_id = service::translate_add_shelf(&state.prosa_client, &request.name, &token.api_key).await?;

    for book in request.items {
        service::translate_add_book_to_shelf(
            &state.prosa_client,
            &shelf_id,
            &book.revision_id,
            &token.api_key,
        )
        .await?;
    }

    Ok((StatusCode::CREATED, shelf_id))
}

pub async fn delete_shelf_handler(
    State(state): State<AppState>,
    Path(shelf_id): Path<String>,
    Extension(token): Extension<AuthToken>,
) -> Result<impl IntoResponse, KoboError> {
    service::translate_delete_shelf(&state.prosa_client, &shelf_id, &token.api_key).await?;

    Ok(())
}

pub async fn rename_shelf_handler(
    State(state): State<AppState>,
    Path(shelf_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<RenameShelfRequest>,
) -> Result<impl IntoResponse, KoboError> {
    service::translate_rename_shelf(&state.prosa_client, &shelf_id, &request.name, &token.api_key).await?;

    Ok(())
}

pub async fn add_book_to_shelf_handler(
    State(state): State<AppState>,
    Path(shelf_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<AddBooksToShelfRequest>,
) -> Result<impl IntoResponse, KoboError> {
    for book in request.items {
        service::translate_add_book_to_shelf(
            &state.prosa_client,
            &shelf_id,
            &book.revision_id,
            &token.api_key,
        )
        .await?;
    }

    Ok(StatusCode::CREATED)
}

pub async fn delete_books_from_shelf_handler(
    State(state): State<AppState>,
    Path(shelf_id): Path<String>,
    Extension(token): Extension<AuthToken>,
    Json(request): Json<DeleteBooksFromShelfRequest>,
) -> Result<impl IntoResponse, KoboError> {
    for book in request.items {
        service::translate_delete_book_from_shelf(
            &state.prosa_client,
            &shelf_id,
            &book.revision_id,
            &token.api_key,
        )
        .await?;
    }

    Ok(())
}
