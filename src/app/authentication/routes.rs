use super::handlers;
use crate::app::AppState;
use axum::{
    routing::{get, post},
    Router,
};

#[rustfmt::skip]
pub fn get_routes(state: AppState) -> Router {
    Router::new()
        .route("/oauth/{device_id}/.well-known/openid-configuration", get(handlers::oauth_configs_handler))
        .route("/oauth/connect/token", post(handlers::oauth_token_handler))
        .with_state(state)
}
