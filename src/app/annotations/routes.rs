use super::handlers;
use crate::app::{authentication::middleware::extract_token_middleware, AppState};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, patch, post},
    Router,
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v3/content/checkforchanges", post(handlers::check_for_changes_handler))
        .route("/api/v3/content/{book_id}/annotations", get(handlers::get_annotations_handler)
            .route_layer(from_fn_with_state(state.clone(), extract_token_middleware))
        )
        .route("/api/v3/content/{book_id}/annotations", patch(handlers::patch_annotations_handler)
            .route_layer(from_fn_with_state(state.clone(), extract_token_middleware))
        )
        .with_state(state)
}
