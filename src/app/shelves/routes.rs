use super::handlers;
use crate::app::{authentication::middleware::extract_token_middleware, AppState};
use axum::{
    middleware::from_fn_with_state,
    routing::{delete, post, put},
    Router,
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/library/tags", post(handlers::create_shelf_handler))
        .route("/v1/library/tags/{shelf_id}", delete(handlers::delete_shelf_handler))
        .route("/v1/library/tags/{shelf_id}", put(handlers::rename_shelf_handler))
        .route("/v1/library/tags/{shelf_id}/items", post(handlers::add_book_to_shelf_handler))
        .route("/v1/library/tags/{shelf_id}/items/delete", post(handlers::delete_books_from_shelf_handler))
        .layer(from_fn_with_state(state.clone(), extract_token_middleware))
        .with_state(state)
}
