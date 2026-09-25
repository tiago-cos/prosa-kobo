use super::handlers;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get},
};

#[rustfmt::skip]
pub fn get_routes() -> Router {
    Router::new()
        .route("/books/{book_id}", get(handlers::download_book_handler))
        .route("/v1/library/{book_id}", delete(handlers::delete_book_handler))
        .layer(DefaultBodyLimit::max(31457280))
}
