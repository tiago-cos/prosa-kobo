mod common;

use axum::{
    body::Body,
    http::{Method, Response, StatusCode},
};
use base64::{
    Engine,
    prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD},
};
use common::{
    API_KEY, EVERY_CAPABILITY, HOST, Harness, USER, assert_internal_error, bearer, body_json, expired_jwt,
    jwt,
};
use prosa_kobo::client::{mock::ProsaMethod, prosa::ClientError};
use serde_json::{Value, json};

const OTHER: &str = "someone-else";

async fn link(harness: &Harness, token: &str, body: Value) -> Response<Body> {
    let auth = bearer(token);

    harness
        .json_with(Method::POST, "/devices/linked", &[("Authorization", &auth)], body)
        .await
}

async fn list(harness: &Harness, token: &str, query: &str) -> Response<Body> {
    let auth = bearer(token);

    harness
        .request(
            Method::GET,
            &format!("/devices/linked{query}"),
            &[("Authorization", &auth)],
        )
        .await
}

async fn unlink(harness: &Harness, token: &str, device_id: &str) -> Response<Body> {
    let auth = bearer(token);

    harness
        .request(
            Method::DELETE,
            &format!("/devices/linked/{device_id}"),
            &[("Authorization", &auth)],
        )
        .await
}

fn user() -> String {
    jwt(USER, false, &EVERY_CAPABILITY)
}

fn admin() -> String {
    jwt("an-admin", true, &EVERY_CAPABILITY)
}

fn kobo(name: &str) -> Value {
    json!({ "name": name, "api_key": API_KEY })
}

async fn linked(harness: &Harness) -> Value {
    harness.recognize(API_KEY);
    let response = link(harness, &user(), kobo("Kobo")).await;
    assert_eq!(response.status(), StatusCode::OK);

    body_json(response).await
}

fn lookup_key(linked: &Value) -> String {
    linked["api_endpoint"]
        .as_str()
        .and_then(|endpoint| endpoint.rsplit('/').next())
        .expect("The endpoint should end in a lookup key")
        .to_owned()
}

#[tokio::test]
async fn links_a_device_and_hands_back_an_endpoint_it_can_use() {
    let harness = Harness::new().await;

    let linked = linked(&harness).await;
    let endpoint = linked["api_endpoint"]
        .as_str()
        .expect("api_endpoint should be a string");

    assert!(endpoint.starts_with(&format!("http://{HOST}/")));
    assert_eq!(
        harness
            .get(&format!("/{}/v1/initialization", lookup_key(&linked)))
            .await
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn hands_each_device_a_key_of_its_own() {
    let harness = Harness::new().await;

    let first = linked(&harness).await;
    let second = linked(&harness).await;

    assert_ne!(first["device_id"], second["device_id"]);
    assert_ne!(lookup_key(&first), lookup_key(&second));
}

#[tokio::test]
async fn refuses_an_api_key_prosa_does_not_recognize() {
    let harness = Harness::new().await;

    let response = link(&harness, &user(), kobo("Kobo")).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["error_code"], "InvalidApiKey");
}

#[tokio::test]
async fn refuses_an_api_key_prosa_says_cannot_be_used() {
    let harness = Harness::new().await;
    harness.client.fail(ProsaMethod::Identity, ClientError::Forbidden);

    let response = link(&harness, &user(), kobo("Kobo")).await;

    assert_eq!(body_json(response).await["error_code"], "InsufficientApiKey");
}

#[tokio::test]
async fn refuses_a_malformed_api_key_without_asking_prosa() {
    let harness = Harness::new().await;

    let response = link(
        &harness,
        &user(),
        json!({ "name": "Kobo", "api_key": "not a key!" }),
    )
    .await;

    assert_eq!(body_json(response).await["error_code"], "InvalidApiKey");
    assert_eq!(harness.client.call_count(ProsaMethod::Identity), 0);
}

#[tokio::test]
async fn refuses_a_device_without_a_name() {
    let harness = Harness::new().await;
    harness.recognize(API_KEY);

    let response = link(&harness, &user(), kobo("   ")).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["error_code"], "InvalidDeviceName");
}

#[tokio::test]
async fn links_a_device_only_with_permission_to_create() {
    let harness = Harness::new().await;
    harness.recognize(API_KEY);

    let response = link(
        &harness,
        &jwt(USER, false, &["Read", "Update", "Delete"]),
        kobo("Kobo"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn will_not_link_a_device_for_someone_else() {
    let harness = Harness::new().await;
    harness.recognize(API_KEY);

    let body = json!({ "name": "Kobo", "api_key": API_KEY, "user_id": OTHER });
    let response = link(&harness, &user(), body).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn lets_an_admin_link_a_device_for_someone_else() {
    let harness = Harness::new().await;
    harness.recognize(API_KEY);

    let body = json!({ "name": "Kobo", "api_key": API_KEY, "user_id": OTHER });
    let response = link(&harness, &admin(), body).await;
    let theirs = body_json(list(&harness, &admin(), &format!("?user_id={OTHER}")).await).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(theirs.as_array().map(Vec::len), Some(1));
    assert_eq!(theirs[0]["user_id"], OTHER);
}

#[tokio::test]
async fn lists_a_users_devices_without_their_secrets() {
    let harness = Harness::new().await;
    let linked = linked(&harness).await;

    let response = list(&harness, &user(), &format!("?user_id={USER}")).await;
    let devices = body_json(response).await;

    assert_eq!(
        devices,
        json!([{ "device_id": linked["device_id"], "user_id": USER, "name": "Kobo" }])
    );
}

#[tokio::test]
async fn lists_only_the_devices_of_the_user_asked_about() {
    let harness = Harness::new().await;
    linked(&harness).await;
    harness.recognize(API_KEY);
    link(
        &harness,
        &admin(),
        json!({ "name": "Theirs", "api_key": API_KEY, "user_id": OTHER }),
    )
    .await;

    let mine = body_json(list(&harness, &user(), &format!("?user_id={USER}")).await).await;

    assert_eq!(mine.as_array().map(Vec::len), Some(1));
    assert_eq!(mine[0]["name"], "Kobo");
}

#[tokio::test]
async fn will_not_list_devices_without_saying_whose() {
    let harness = Harness::new().await;
    linked(&harness).await;

    assert_eq!(list(&harness, &user(), "").await.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn will_not_list_someone_elses_devices() {
    let harness = Harness::new().await;

    let response = list(&harness, &user(), &format!("?user_id={OTHER}")).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn lets_an_admin_list_every_device() {
    let harness = Harness::new().await;
    linked(&harness).await;
    harness.recognize(API_KEY);
    link(
        &harness,
        &admin(),
        json!({ "name": "Theirs", "api_key": API_KEY, "user_id": OTHER }),
    )
    .await;

    let every = body_json(list(&harness, &admin(), "").await).await;

    assert_eq!(every.as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn lists_devices_only_with_permission_to_read() {
    let harness = Harness::new().await;

    let response = list(
        &harness,
        &jwt(USER, false, &["Create"]),
        &format!("?user_id={USER}"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn unlinking_a_device_retires_its_endpoint() {
    let harness = Harness::new().await;
    let linked = linked(&harness).await;
    let device_id = linked["device_id"]
        .as_str()
        .expect("device_id should be a string");

    let response = unlink(&harness, &user(), device_id).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        harness
            .get(&format!("/{}/v1/initialization", lookup_key(&linked)))
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn will_not_unlink_someone_elses_device() {
    let harness = Harness::new().await;
    let linked = linked(&harness).await;
    let device_id = linked["device_id"]
        .as_str()
        .expect("device_id should be a string");

    let response = unlink(&harness, &jwt(OTHER, false, &EVERY_CAPABILITY), device_id).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        harness
            .get(&format!("/{}/v1/initialization", lookup_key(&linked)))
            .await
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn lets_an_admin_unlink_anyones_device() {
    let harness = Harness::new().await;
    let linked = linked(&harness).await;
    let device_id = linked["device_id"]
        .as_str()
        .expect("device_id should be a string");

    assert_eq!(
        unlink(&harness, &admin(), device_id).await.status(),
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn answers_not_found_for_a_device_that_was_never_linked() {
    let harness = Harness::new().await;

    let response = unlink(&harness, &user(), "nothing").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn unlinks_only_with_permission_to_delete() {
    let harness = Harness::new().await;
    let linked = linked(&harness).await;
    let device_id = linked["device_id"]
        .as_str()
        .expect("device_id should be a string");

    let response = unlink(
        &harness,
        &jwt(USER, false, &["Read", "Create", "Update"]),
        device_id,
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn turns_away_a_request_without_a_prosa_token() {
    let harness = Harness::new().await;

    let response = harness
        .request(Method::GET, &format!("/devices/linked?user_id={USER}"), &[])
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn turns_away_an_expired_prosa_token() {
    let harness = Harness::new().await;

    let response = list(&harness, &expired_jwt(USER), &format!("?user_id={USER}")).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn turns_away_something_that_is_not_a_token() {
    let harness = Harness::new().await;

    let response = list(&harness, "bm90IGEgdG9rZW4=", &format!("?user_id={USER}")).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn turns_away_a_token_whose_claims_were_altered() {
    let harness = Harness::new().await;
    linked(&harness).await;

    let genuine = String::from_utf8(
        BASE64_STANDARD
            .decode(user())
            .expect("The token should be base64"),
    )
    .expect("The token should be text");
    let [header, _, signature] = genuine.split('.').collect::<Vec<_>>()[..] else {
        panic!("The token should have three parts");
    };
    let promoted = BASE64_URL_SAFE_NO_PAD.encode(
        json!({ "role": { "Admin": USER }, "capabilities": EVERY_CAPABILITY, "exp": u64::MAX,
                "session_id": "session", "iss": "prosa" })
        .to_string(),
    );
    let forged = BASE64_STANDARD.encode(format!("{header}.{promoted}.{signature}"));

    let response = list(&harness, &forged, "").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn answers_an_internal_error_when_the_device_cannot_be_stored() {
    let harness = Harness::new().await;
    harness.recognize(API_KEY);
    harness.fail_writes_to("linked_devices").await;

    assert_internal_error(link(&harness, &user(), kobo("Kobo")).await).await;
}
