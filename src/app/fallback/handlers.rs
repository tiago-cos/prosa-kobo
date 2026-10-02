use crate::app::tracing::Unhandled;
use axum::{
    Extension,
    http::{Method, StatusCode},
    response::IntoResponse,
};

pub async fn fallback_handler(method: Method) -> impl IntoResponse {
    let status = if method == Method::PATCH {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    };

    (Extension(Unhandled), status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn marks_its_answer_as_unhandled() {
        for method in [Method::GET, Method::PATCH] {
            let response = fallback_handler(method).await.into_response();

            assert!(response.extensions().get::<Unhandled>().is_some());
        }
    }
}
