mod common;

use axum::http::{Request, StatusCode};
use common::{Harness, body_json};
use serde_json::json;

#[tokio::test]
async fn the_router_answers_without_a_prosa_behind_it() {
    let harness = Harness::new().await;

    let response = harness.get("/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_json(response).await,
        json!({ "status": "ok", "software": "prosa-kobo", "version": env!("CARGO_PKG_VERSION") })
    );
}

#[tokio::test]
async fn a_request_under_an_unknown_key_is_turned_away() {
    let harness = Harness::new().await;
    let response = harness.get("/notakey/v1/library/sync").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_device_route_answers_rather_than_reaching_for_prosa() {
    let harness = Harness::new().await;

    let request = Request::builder()
        .uri("/devices/linked")
        .header("Authorization", "Bearer notatoken")
        .body(axum::body::Body::empty())
        .expect("Failed to build the request");

    assert_eq!(harness.send(request).await.status(), StatusCode::UNAUTHORIZED);
}
