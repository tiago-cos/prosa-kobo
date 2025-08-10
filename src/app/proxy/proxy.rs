use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
};

pub async fn proxy_handler(method: Method) -> impl IntoResponse {
    if method == Method::PATCH {
        return StatusCode::NO_CONTENT;
    }

    StatusCode::NOT_FOUND
}
