use super::handlers;
use crate::app::AppState;
use axum::{
    Router,
    routing::{get, patch, post},
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v3/content/checkforchanges", post(handlers::check_for_changes_handler))
        .route("/api/v3/content/{book_id}/annotations", get(handlers::get_annotations_handler))
        .route("/api/v3/content/{book_id}/annotations", patch(handlers::patch_annotations_handler))
        .with_state(state)
}
