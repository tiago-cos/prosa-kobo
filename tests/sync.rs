mod common;

use axum::http::{Method, StatusCode};
use common::{Device, Harness, assert_internal_error, body_json};
use prosa_kobo::client::{
    ProsaReadingStatus,
    mock::ProsaMethod,
    prosa::ClientError,
    state::{ProsaState, ProsaStatistics},
    sync::{ProsaBookSync, ProsaShelfSync, ProsaSync},
};
use serde_json::{Value, json};

const TOKEN: &str = "X-Kobo-Synctoken";

fn books(change: impl FnOnce(&mut ProsaBookSync)) -> ProsaSync {
    let mut sync = ProsaSync {
        new_sync_token: 7,
        ..ProsaSync::default()
    };
    change(&mut sync.unsynced_books);

    sync
}

fn shelves(change: impl FnOnce(&mut ProsaShelfSync)) -> ProsaSync {
    let mut sync = ProsaSync {
        new_sync_token: 7,
        ..ProsaSync::default()
    };
    change(&mut sync.unsynced_shelves);

    sync
}

async fn sync(harness: &Harness, device: &Device) -> Vec<Value> {
    let response = harness.get(&device.at("/v1/library/sync")).await;
    assert_eq!(response.status(), StatusCode::OK);

    body_json(response)
        .await
        .as_array()
        .expect("A sync answers a list")
        .clone()
}

fn entitlement(item: &Value) -> &Value {
    &item["NewEntitlement"]
}

#[tokio::test]
async fn answers_nothing_when_nothing_changed_and_hands_back_the_next_token() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_sync(books(|_| ()));

    let response = harness.get(&device.at("/v1/library/sync")).await;

    assert_eq!(response.headers()[TOKEN], "7");
    assert_eq!(body_json(response).await, json!([]));
}

#[tokio::test]
async fn asks_prosa_for_what_changed_since_the_token_the_device_holds() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_sync(books(|_| ()));

    harness
        .request(Method::GET, &device.at("/v1/library/sync"), &[(TOKEN, "42")])
        .await;

    let calls = harness.client.calls_to(ProsaMethod::SyncDevice);

    assert_eq!(calls[0].arguments, vec!["42".to_owned()]);
}

#[tokio::test]
async fn sends_a_new_book_with_its_metadata_and_reading_state() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book("book");
    harness
        .client
        .seed_sync(books(|books| books.file = vec!["book".to_owned()]));

    let items = sync(&harness, &device).await;
    let entitlement = entitlement(&items[0]);

    assert_eq!(items.len(), 1);
    assert_eq!(entitlement["BookEntitlement"]["Id"], "book");
    assert_eq!(entitlement["BookEntitlement"]["IsRemoved"], false);
    assert_eq!(entitlement["BookMetadata"]["EntitlementId"], "book");
    assert_eq!(entitlement["ReadingState"]["EntitlementId"], "book");
}

fn reading(location: Option<&str>) -> ProsaState {
    ProsaState {
        location: location.map(str::to_owned),
        statistics: ProsaStatistics {
            rating: None,
            reading_status: ProsaReadingStatus::Reading,
        },
    }
}

#[tokio::test]
async fn sends_a_reading_state_changed_elsewhere_on_its_own() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book("book");
    harness.client.seed_state("book", reading(None));
    harness
        .client
        .seed_sync(books(|books| books.state = vec!["book".to_owned()]));

    let items = sync(&harness, &device).await;
    let state = &items[0]["ChangedReadingState"]["ReadingState"];

    assert_eq!(items.len(), 1);
    assert!(items[0]["NewEntitlement"].is_null());
    assert_eq!(state["EntitlementId"], "book");
    assert_eq!(state["StatusInfo"]["Status"], "Reading");
}

#[tokio::test]
async fn sends_the_bookmark_as_the_span_the_device_reads() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book("book");
    harness.client.seed_state(
        "book",
        reading(Some("OEBPS/7860148755851063127_64317-h-2.htm.xhtml#0/0/0/t0:0")),
    );
    harness
        .client
        .seed_sync(books(|books| books.state = vec!["book".to_owned()]));

    let items = sync(&harness, &device).await;
    let location = &items[0]["ChangedReadingState"]["ReadingState"]["CurrentBookmark"]["Location"];

    assert_eq!(location["Value"], "kobo.1.1");
    assert_eq!(location["Type"], "KoboSpan");
}

#[tokio::test]
async fn folds_a_changed_reading_state_into_a_book_already_being_sent() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book("book");
    harness.client.seed_state("book", reading(None));
    harness.client.seed_sync(books(|books| {
        books.metadata = vec!["book".to_owned()];
        books.state = vec!["book".to_owned()];
    }));

    let items = sync(&harness, &device).await;

    assert_eq!(items.len(), 1);
    assert_eq!(
        entitlement(&items[0])["ReadingState"]["StatusInfo"]["Status"],
        "Reading"
    );
}

#[tokio::test]
async fn sends_no_reading_state_for_a_book_being_removed() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_sync(books(|books| {
        books.deleted = vec!["book".to_owned()];
        books.state = vec!["book".to_owned()];
    }));

    let items = sync(&harness, &device).await;

    assert_eq!(items.len(), 1);
    assert_eq!(entitlement(&items[0])["BookEntitlement"]["IsRemoved"], true);
}

#[tokio::test]
async fn sends_a_book_changed_several_ways_once() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book("book");
    harness.client.seed_sync(books(|books| {
        books.file = vec!["book".to_owned()];
        books.metadata = vec!["book".to_owned()];
        books.cover = vec!["book".to_owned()];
    }));

    assert_eq!(sync(&harness, &device).await.len(), 1);
}

#[tokio::test]
async fn tells_the_device_to_remove_a_deleted_book() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness
        .client
        .seed_sync(books(|books| books.deleted = vec!["book".to_owned()]));

    let items = sync(&harness, &device).await;

    assert_eq!(entitlement(&items[0])["BookEntitlement"]["Id"], "book");
    assert_eq!(entitlement(&items[0])["BookEntitlement"]["IsRemoved"], true);
}

#[tokio::test]
async fn moves_the_cover_id_when_the_cover_changed() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book("book");

    let before =
        body_json(harness.get(&device.at("/v1/library/book/metadata")).await).await[0]["CoverImageId"]
            .clone();

    harness
        .client
        .seed_sync(books(|books| books.cover = vec!["book".to_owned()]));
    let items = sync(&harness, &device).await;
    let after = entitlement(&items[0])["BookMetadata"]["CoverImageId"].clone();

    assert_ne!(before, after);
}

#[tokio::test]
async fn converts_a_book_again_once_its_file_changed() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book("book");
    harness.get(&device.at("/books/book")).await;

    harness
        .client
        .seed_sync(books(|books| books.file = vec!["book".to_owned()]));
    sync(&harness, &device).await;

    assert_eq!(harness.client.call_count(ProsaMethod::DownloadBook), 2);
}

#[tokio::test]
async fn reports_a_book_whose_annotations_changed_when_the_device_next_checks() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    let check = json!([{ "ContentId": "book", "etag": "held" }]);

    harness
        .json(
            Method::POST,
            &device.at("/api/v3/content/checkforchanges"),
            check.clone(),
        )
        .await;

    harness
        .client
        .seed_sync(books(|books| books.annotations = vec!["book".to_owned()]));
    sync(&harness, &device).await;

    let changed = harness
        .json(Method::POST, &device.at("/api/v3/content/checkforchanges"), check)
        .await;

    assert_eq!(body_json(changed).await, json!(["book"]));
}

#[tokio::test]
async fn sends_a_changed_shelf_with_its_name_and_books() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness
        .client
        .seed_shelf("shelf", "To read", &["first", "second"]);
    harness
        .client
        .seed_sync(shelves(|shelves| shelves.contents = vec!["shelf".to_owned()]));

    let items = sync(&harness, &device).await;
    let tag = &items[0]["NewTag"]["Tag"];

    assert_eq!(tag["Id"], "shelf");
    assert_eq!(tag["Name"], "To read");
    assert_eq!(tag["Items"].as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn tells_the_device_to_drop_a_deleted_shelf() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness
        .client
        .seed_sync(shelves(|shelves| shelves.deleted = vec!["shelf".to_owned()]));

    let items = sync(&harness, &device).await;

    assert_eq!(items[0]["DeletedTag"]["Tag"]["Id"], "shelf");
}

#[tokio::test]
async fn passes_on_prosa_failing_the_sync() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness
        .client
        .fail(ProsaMethod::SyncDevice, ClientError::Forbidden);

    let response = harness.get(&device.at("/v1/library/sync")).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn answers_an_internal_error_when_the_cover_version_cannot_be_moved() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book("book");
    harness
        .client
        .seed_sync(books(|books| books.cover = vec!["book".to_owned()]));
    harness.fail_writes_to("cover_versions").await;

    assert_internal_error(harness.get(&device.at("/v1/library/sync")).await).await;
}
