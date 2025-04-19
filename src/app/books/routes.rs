use super::handlers;
use crate::app::{authentication::middleware::extract_token_middleware, AppState};
use axum::{
    extract::DefaultBodyLimit,
    middleware::from_fn_with_state,
    routing::{delete, get},
    Router,
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/books/{book_id}", get(handlers::download_book_handler))
        .route("/v1/library/{book_id}", delete(handlers::delete_book_handler)
            .route_layer(from_fn_with_state(state.clone(), extract_token_middleware))
        )
        .layer(DefaultBodyLimit::max(31457280))
        .with_state(state)
}
