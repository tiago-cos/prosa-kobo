use super::handlers;
use axum::{
    Router,
    routing::{get, post},
};

#[rustfmt::skip]
pub fn get_routes() -> Router {
    Router::new()
        .route("/oauth/.well-known/openid-configuration", get(handlers::oauth_configs_handler))
        .route("/oauth/connect/token", post(handlers::oauth_token_handler))
        .route("/v1/auth/device", post(handlers::device_auth_handler))
        .route("/v1/auth/refresh", post(handlers::device_auth_handler))
}
