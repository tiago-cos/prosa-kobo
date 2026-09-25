use super::handlers;
use axum::{
    Router,
    routing::{delete, post, put},
};

#[rustfmt::skip]
pub fn get_routes() -> Router {
    Router::new()
        .route("/v1/library/tags", post(handlers::create_shelf_handler))
        .route("/v1/library/tags/{shelf_id}", delete(handlers::delete_shelf_handler))
        .route("/v1/library/tags/{shelf_id}", put(handlers::rename_shelf_handler))
        .route("/v1/library/tags/{shelf_id}/items", post(handlers::add_book_to_shelf_handler))
        .route("/v1/library/tags/{shelf_id}/items/delete", post(handlers::delete_books_from_shelf_handler))
}
