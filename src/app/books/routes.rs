use super::handlers;
use crate::app::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get},
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/books/{book_id}", get(handlers::download_book_handler))
        .route("/v1/library/{book_id}", delete(handlers::delete_book_handler))
        .layer(DefaultBodyLimit::max(31457280))
        .with_state(state)
}
