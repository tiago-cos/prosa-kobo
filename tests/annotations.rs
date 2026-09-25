mod common;

use axum::{
    body::Body,
    http::{Method, Response, StatusCode},
};
use common::{DEVICE_HARDWARE_ID, Harness, assert_internal_error, body_json};
use prosa_kobo::client::{mock::ProsaMethod, prosa::ClientError};
use serde_json::{Value, json};

const BOOK: &str = "book";
const CHAPTER: &str = "OEBPS/7860148755851063127_64317-h-2.htm.xhtml";
const FIRST: &str = "kobo.1.1";
const SECOND: &str = "kobo.2.1";
const HIGHLIGHT: &str = "11111111-1111-4111-8111-111111111111";
const OTHER: &str = "22222222-2222-4222-8222-222222222222";
const DEVICE: [(&str, &str); 1] = [("x-kobo-deviceid", DEVICE_HARDWARE_ID)];

fn selector(span: &str) -> String {
    format!("span#{}", span.replace('.', r"\."))
}

fn annotation(id: &str, start: (&str, u32), end: (&str, u32), note: Option<&str>) -> Value {
    json!({
        "clientLastModifiedUtc": "2026-09-20T16:03:39Z",
        "id": id,
        "location": { "span": {
            "chapterFilename": CHAPTER,
            "startPath": selector(start.0),
            "startChar": start.1,
            "endPath": selector(end.0),
            "endChar": end.1,
        }},
        "noteText": note,
        "type": if note.is_some() { "note" } else { "highlight" },
    })
}

async fn patch(harness: &Harness, updated: Vec<Value>, deleted: Vec<&str>) -> Response<Body> {
    harness
        .json_with(
            Method::PATCH,
            "/api/v3/content/book/annotations",
            &DEVICE,
            json!({ "updatedAnnotations": updated, "deletedAnnotationIds": deleted }),
        )
        .await
}

async fn annotations(harness: &Harness) -> Vec<Value> {
    let response = harness
        .request(Method::GET, "/api/v3/content/book/annotations", &DEVICE)
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    body_json(response).await["annotations"]
        .as_array()
        .expect("annotations should be a list")
        .clone()
}

async fn device_with_book() -> Harness {
    let harness = Harness::new().await;
    harness.introduced().await;
    harness.add_book(BOOK);

    harness
}

#[tokio::test]
async fn stores_a_highlight_as_prosa_locations() {
    let harness = device_with_book().await;

    let response = patch(
        &harness,
        vec![annotation(HIGHLIGHT, (FIRST, 0), (SECOND, 3), None)],
        vec![],
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let stored = harness.client.stored_annotations(BOOK);

    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].annotation_id, HIGHLIGHT);
    assert_eq!(stored[0].start_location, format!("{CHAPTER}#0/0/0/t0:0"));
    assert_eq!(stored[0].end_location, format!("{CHAPTER}#0/1/t0:3"));
}

#[tokio::test]
async fn hands_a_highlight_back_as_the_device_wrote_it() {
    let harness = device_with_book().await;
    patch(
        &harness,
        vec![annotation(HIGHLIGHT, (FIRST, 0), (SECOND, 3), None)],
        vec![],
    )
    .await;

    let returned = annotations(&harness).await;
    let span = &returned[0]["location"]["span"];

    assert_eq!(returned[0]["id"], HIGHLIGHT);
    assert_eq!(returned[0]["type"], "highlight");
    assert_eq!(span["chapterFilename"], CHAPTER);
    assert_eq!(span["startPath"], selector(FIRST));
    assert_eq!(span["startChar"], 0);
    assert_eq!(span["endPath"], selector(SECOND));
    assert_eq!(span["endChar"], 3);
}

#[tokio::test]
async fn keeps_a_one_character_highlight() {
    let harness = device_with_book().await;

    let response = patch(
        &harness,
        vec![annotation(HIGHLIGHT, (FIRST, 5), (FIRST, 6), None)],
        vec![],
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(harness.client.stored_annotations(BOOK).len(), 1);
}

#[tokio::test]
async fn reaches_to_the_end_of_a_span() {
    let harness = device_with_book().await;

    patch(
        &harness,
        vec![annotation(HIGHLIGHT, (FIRST, 0), (FIRST, 59), None)],
        vec![],
    )
    .await;

    assert_eq!(
        harness.client.stored_annotations(BOOK)[0].end_location,
        format!("{CHAPTER}#0/0/0/t0:59")
    );
}

#[tokio::test]
async fn updates_the_note_of_an_annotation_it_already_holds() {
    let harness = device_with_book().await;
    patch(
        &harness,
        vec![annotation(HIGHLIGHT, (FIRST, 0), (SECOND, 3), None)],
        vec![],
    )
    .await;

    patch(
        &harness,
        vec![annotation(
            HIGHLIGHT,
            (FIRST, 0),
            (SECOND, 3),
            Some("A later note"),
        )],
        vec![],
    )
    .await;

    let stored = harness.client.stored_annotations(BOOK);

    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].note.as_deref(), Some("A later note"));
    assert_eq!(annotations(&harness).await[0]["type"], "note");
}

#[tokio::test]
async fn deletes_what_the_device_removed() {
    let harness = device_with_book().await;
    patch(
        &harness,
        vec![annotation(HIGHLIGHT, (FIRST, 0), (SECOND, 3), None)],
        vec![],
    )
    .await;

    let response = patch(&harness, vec![], vec![HIGHLIGHT]).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(harness.client.stored_annotations(BOOK).is_empty());
}

#[tokio::test]
async fn drops_only_the_annotation_the_book_cannot_place() {
    let harness = device_with_book().await;

    let response = patch(
        &harness,
        vec![
            annotation(HIGHLIGHT, ("kobo.99999.1", 0), ("kobo.99999.1", 3), None),
            annotation(OTHER, (FIRST, 0), (SECOND, 3), None),
        ],
        vec![],
    )
    .await;

    let stored = harness.client.stored_annotations(BOOK);

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].annotation_id, OTHER);
}

#[tokio::test]
async fn lists_an_unannotated_book_without_converting_it() {
    let harness = device_with_book().await;

    assert!(annotations(&harness).await.is_empty());
    assert_eq!(harness.client.call_count(ProsaMethod::DownloadBook), 0);
}

#[tokio::test]
async fn reports_a_book_only_once_the_device_holds_a_stale_etag() {
    let harness = device_with_book().await;

    let response = harness
        .request(Method::GET, "/api/v3/content/book/annotations", &DEVICE)
        .await;
    let etag = response.headers()["ETag"]
        .to_str()
        .expect("The ETag should be text")
        .to_owned();

    let check = |etag: &str| json!([{ "ContentId": BOOK, "etag": etag }]);
    let current = harness
        .json_with(
            Method::POST,
            "/api/v3/content/checkforchanges",
            &DEVICE,
            check(&etag),
        )
        .await;
    let stale = harness
        .json_with(
            Method::POST,
            "/api/v3/content/checkforchanges",
            &DEVICE,
            check("stale"),
        )
        .await;

    assert_eq!(body_json(current).await, json!([]));
    assert_eq!(body_json(stale).await, json!([BOOK]));
}

#[tokio::test]
async fn answers_not_found_for_a_book_prosa_does_not_hold() {
    let harness = Harness::new().await;
    harness.introduced().await;

    let response = harness
        .request(Method::GET, "/api/v3/content/book/annotations", &DEVICE)
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn passes_on_prosa_refusing_the_key() {
    let harness = device_with_book().await;
    harness
        .client
        .fail(ProsaMethod::AddAnnotation, ClientError::Forbidden);

    let response = patch(
        &harness,
        vec![annotation(HIGHLIGHT, (FIRST, 0), (SECOND, 3), None)],
        vec![],
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn answers_an_internal_error_when_the_etag_cannot_be_stored() {
    let harness = device_with_book().await;
    harness.fail_writes_to("etags").await;

    let response = harness
        .request(Method::GET, "/api/v3/content/book/annotations", &DEVICE)
        .await;

    assert_internal_error(response).await;
}
