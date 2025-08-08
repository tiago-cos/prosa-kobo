use super::handlers;
use crate::app::AppState;
use axum::{Router, extract::DefaultBodyLimit, routing::get};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/images/{book_id}", get(handlers::download_cover_handler))
        .layer(DefaultBodyLimit::max(31457280))
        .with_state(state)
}
