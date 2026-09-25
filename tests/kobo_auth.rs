mod common;

use axum::http::{Method, StatusCode};
use common::{API_KEY, DEVICE_HARDWARE_ID, Harness, assert_internal_error, body_json};

const DEVICE_ID: &str = "x-kobo-deviceid";

#[tokio::test]
async fn a_request_under_the_lookup_key_reaches_the_route_it_names() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness.get(&device.at("/v1/initialization")).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_request_under_an_unknown_lookup_key_is_turned_away() {
    let harness = Harness::new().await;
    harness.linked().await;

    let response = harness.get("/notthekey/v1/initialization").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_request_carrying_no_key_at_all_is_turned_away() {
    let harness = Harness::new().await;
    harness.linked().await;

    let response = harness.get("/v1/initialization").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unlinking_a_device_retires_its_key() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    assert_eq!(
        harness.get(&device.at("/v1/initialization")).await.status(),
        StatusCode::OK
    );

    harness.unlink(&device.device_id).await;

    assert_eq!(
        harness.get(&device.at("/v1/initialization")).await.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn the_resources_it_is_given_carry_its_key() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness.get(&device.at("/v1/initialization")).await;
    let resources = body_json(response).await;
    let resources = &resources["Resources"];

    for name in ["library_sync", "book", "image_host", "oauth_host"] {
        let url = resources[name]
            .as_str()
            .unwrap_or_else(|| panic!("{name} should be a URL"));

        assert!(
            url.contains(&device.lookup_key),
            "{name} should carry the lookup key, got {url}"
        );
    }
}

#[tokio::test]
async fn an_unkeyed_reading_services_call_is_turned_away_before_the_device_is_known() {
    let harness = Harness::new().await;
    harness.linked().await;

    let response = harness
        .request(
            Method::GET,
            "/api/v3/content/book/annotations",
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_keyed_call_teaches_the_middleware_the_hardware_id() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_book("book");

    harness
        .request(
            Method::GET,
            &device.at("/v1/initialization"),
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    let response = harness
        .request(
            Method::GET,
            "/api/v3/content/book/annotations",
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn an_unkeyed_call_naming_another_hardware_id_is_turned_away() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    harness
        .request(
            Method::GET,
            &device.at("/v1/initialization"),
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    let response = harness
        .request(
            Method::GET,
            "/api/v3/content/book/annotations",
            &[(DEVICE_ID, "someoneelse")],
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_unkeyed_call_with_no_hardware_id_is_turned_away() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    harness
        .request(
            Method::GET,
            &device.at("/v1/initialization"),
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    let response = harness.get("/api/v3/content/book/annotations").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn relinking_the_same_hardware_moves_its_id_to_the_new_device() {
    let harness = Harness::new().await;
    let first = harness.linked().await;

    harness
        .request(
            Method::GET,
            &first.at("/v1/initialization"),
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    let second = harness.link("Kobo again", API_KEY).await;
    harness
        .request(
            Method::GET,
            &second.at("/v1/initialization"),
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    harness.client.seed_book("book");

    let response = harness
        .request(
            Method::GET,
            "/api/v3/content/book/annotations",
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn the_oauth_endpoints_answer_so_the_device_proceeds() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let configs = harness
        .get(&device.at("/oauth/.well-known/openid-configuration"))
        .await;

    assert_eq!(configs.status(), StatusCode::OK);

    let endpoint = body_json(configs).await["token_endpoint"]
        .as_str()
        .expect("token_endpoint should be a URL")
        .to_owned();

    assert!(endpoint.contains(&device.lookup_key));

    for path in ["/oauth/connect/token", "/v1/auth/device", "/v1/auth/refresh"] {
        let response = harness
            .json(Method::POST, &device.at(path), serde_json::json!({}))
            .await;

        assert_eq!(response.status(), StatusCode::OK, "{path} should answer");
    }
}

#[tokio::test]
async fn a_keyed_call_whose_hardware_id_cannot_be_recorded_answers_an_internal_error() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.fail_writes_to("linked_devices").await;

    let response = harness
        .request(
            Method::GET,
            &device.at("/v1/initialization"),
            &[(DEVICE_ID, DEVICE_HARDWARE_ID)],
        )
        .await;

    assert_internal_error(response).await;
}
