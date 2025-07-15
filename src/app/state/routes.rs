use super::handlers;
use crate::app::{authentication::middleware::extract_token_middleware, AppState};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put},
    Router,
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/v1/library/{book_id}/state", get(handlers::get_state_handler))
        .route("/v1/library/{book_id}/state", put(handlers::update_state_handler))
        .route("/v1/user/reviews", get(handlers::get_rating_handler))
        .route("/v1/products/{book_id}/rating/{rating}", post(handlers::update_rating_handler))
        .route("/v1/products/{book_id}/reviews", get(handlers::get_reviews_mock_handler))
        .layer(from_fn_with_state(state.clone(), extract_token_middleware))
        .with_state(state)
}
