use super::handlers;
use crate::app::AppState;
use axum::{extract::DefaultBodyLimit, routing::get, Router};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/books/{book_id}", get(handlers::download_book_handler))
        .layer(DefaultBodyLimit::max(31457280))
        .with_state(state)
}
