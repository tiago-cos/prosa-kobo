mod common;

use axum::http::{Method, StatusCode};
use common::{Device, EPOCH, Harness, body_json, reported_state};
use prosa_kobo::client::{
    ProsaReadingStatus,
    mock::ProsaMethod,
    prosa::ClientError,
    state::{ProsaState, ProsaStatistics},
};
use serde_json::{Value, json};

const BOOK: &str = "book";
const CHAPTER: &str = "OEBPS/7860148755851063127_64317-h-2.htm.xhtml";

fn state(location: Option<&str>, reading_status: ProsaReadingStatus) -> ProsaState {
    ProsaState {
        location: location.map(str::to_owned),
        statistics: ProsaStatistics {
            rating: None,
            reading_status,
        },
    }
}

fn update(status: &str, span: Option<&str>) -> Value {
    reported_state(BOOK, CHAPTER, status, span)
}

async fn fetch(harness: &Harness, device: &Device) -> Value {
    let response = harness.get(&device.at("/v1/library/book/state")).await;
    assert_eq!(response.status(), StatusCode::OK);

    body_json(response).await[0].clone()
}

async fn with_book() -> (Harness, Device) {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book(BOOK);

    (harness, device)
}

#[tokio::test]
async fn names_each_reading_status_the_way_the_device_does() {
    let (harness, device) = with_book().await;

    for (prosa, kobo) in [
        (ProsaReadingStatus::Unread, "ReadyToRead"),
        (ProsaReadingStatus::Reading, "Reading"),
        (ProsaReadingStatus::Read, "Finished"),
    ] {
        harness.client.seed_state(BOOK, state(None, prosa));

        assert_eq!(fetch(&harness, &device).await["StatusInfo"]["Status"], kobo);
    }
}

#[tokio::test]
async fn hands_back_the_bookmark_as_the_span_it_falls_in() {
    let (harness, device) = with_book().await;
    harness.client.seed_state(
        BOOK,
        state(Some(&format!("{CHAPTER}#0/1/t0:0")), ProsaReadingStatus::Reading),
    );

    let location = &fetch(&harness, &device).await["CurrentBookmark"]["Location"];

    assert_eq!(location["Value"], "kobo.2.1");
    assert_eq!(location["Type"], "KoboSpan");
    assert_eq!(location["Source"], CHAPTER);
}

#[tokio::test]
async fn hands_back_no_bookmark_when_prosa_holds_none() {
    let (harness, device) = with_book().await;

    assert!(fetch(&harness, &device).await["CurrentBookmark"]["Location"].is_null());
}

#[tokio::test]
async fn hands_back_no_bookmark_rather_than_one_the_book_cannot_place() {
    let (harness, device) = with_book().await;
    harness.client.seed_state(
        BOOK,
        state(Some("OEBPS/nowhere.xhtml#0/t0:1"), ProsaReadingStatus::Reading),
    );

    assert!(fetch(&harness, &device).await["CurrentBookmark"]["Location"].is_null());
}

#[tokio::test]
async fn stores_the_bookmark_the_device_reports_as_a_prosa_location() {
    let (harness, device) = with_book().await;

    let response = harness
        .json(
            Method::PUT,
            &device.at("/v1/library/book/state"),
            update("Reading", Some("kobo.2.1")),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let reply = body_json(response).await;
    assert_eq!(reply["RequestResult"], "Success");
    assert_eq!(reply["UpdateResults"][0]["EntitlementId"], BOOK);

    let stored = harness
        .client
        .stored_state(BOOK)
        .expect("A state should be stored");

    assert_eq!(stored.location, Some(format!("{CHAPTER}#0/1/t0:0")));
    assert_eq!(stored.statistics.reading_status, ProsaReadingStatus::Reading);
}

#[tokio::test]
async fn stores_each_reading_status_the_way_prosa_does() {
    let (harness, device) = with_book().await;

    for (kobo, prosa) in [
        ("ReadyToRead", ProsaReadingStatus::Unread),
        ("Reading", ProsaReadingStatus::Reading),
        ("Finished", ProsaReadingStatus::Read),
    ] {
        harness
            .json(
                Method::PUT,
                &device.at("/v1/library/book/state"),
                update(kobo, None),
            )
            .await;

        let stored = harness
            .client
            .stored_state(BOOK)
            .expect("A state should be stored");

        assert_eq!(stored.statistics.reading_status, prosa);
    }
}

#[tokio::test]
async fn drops_the_position_when_the_device_puts_a_book_aside() {
    let (harness, device) = with_book().await;

    for status in ["Finished", "ReadyToRead"] {
        harness.client.seed_state(
            BOOK,
            ProsaState {
                statistics: ProsaStatistics {
                    rating: Some(4.0),
                    reading_status: ProsaReadingStatus::Reading,
                },
                ..state(Some(&format!("{CHAPTER}#0/1/t0:0")), ProsaReadingStatus::Reading)
            },
        );

        let response = harness
            .json(
                Method::PUT,
                &device.at("/v1/library/book/state"),
                update(status, Some("kobo.1.1")),
            )
            .await;
        let stored = harness
            .client
            .stored_state(BOOK)
            .expect("A state should be stored");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(stored.location, None);
        assert_eq!(stored.statistics.rating, Some(4.0));
    }
    assert_eq!(harness.client.call_count(ProsaMethod::PatchState), 0);
}

#[tokio::test]
async fn keeps_the_position_prosa_holds_over_a_bookmark_the_book_cannot_place() {
    let (harness, device) = with_book().await;
    harness.client.seed_state(
        BOOK,
        state(Some(&format!("{CHAPTER}#0/1/t0:0")), ProsaReadingStatus::Reading),
    );

    let response = harness
        .json(
            Method::PUT,
            &device.at("/v1/library/book/state"),
            update("Reading", Some("kobo.99999.1")),
        )
        .await;

    let stored = harness
        .client
        .stored_state(BOOK)
        .expect("A state should be stored");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(stored.statistics.reading_status, ProsaReadingStatus::Reading);
    assert_eq!(stored.location, Some(format!("{CHAPTER}#0/1/t0:0")));
}

#[tokio::test]
async fn dates_a_state_the_device_already_holds_before_its_own() {
    let (harness, device) = with_book().await;
    harness
        .report_state(&device, BOOK, update("Reading", Some("kobo.2.1")))
        .await;

    let state = fetch(&harness, &device).await;

    assert_eq!(state["CurrentBookmark"]["Location"]["Value"], "kobo.2.1");
    for part in ["StatusInfo", "Statistics", "CurrentBookmark"] {
        assert_eq!(state[part]["LastModified"], EPOCH, "{part}");
    }
    assert_eq!(state["LastModified"], EPOCH);
}

#[tokio::test]
async fn dates_a_position_changed_elsewhere_now() {
    let (harness, device) = with_book().await;
    harness
        .report_state(&device, BOOK, update("Reading", Some("kobo.2.1")))
        .await;
    harness.client.seed_state(
        BOOK,
        state(
            Some(&format!("{CHAPTER}#0/0/0/t0:0")),
            ProsaReadingStatus::Reading,
        ),
    );

    let bookmark = &fetch(&harness, &device).await["CurrentBookmark"];

    assert_eq!(bookmark["Location"]["Value"], "kobo.1.1");
    assert_ne!(bookmark["LastModified"], EPOCH);
}

#[tokio::test]
async fn does_not_offer_back_a_book_the_device_put_aside() {
    let (harness, device) = with_book().await;
    harness
        .report_state(&device, BOOK, update("ReadyToRead", Some("kobo.2.1")))
        .await;

    let state = fetch(&harness, &device).await;

    assert_eq!(state["StatusInfo"]["Status"], "ReadyToRead");
    assert_eq!(state["CurrentBookmark"]["LastModified"], EPOCH);
}

#[tokio::test]
async fn dates_a_state_now_for_a_device_that_never_reported_it() {
    let (harness, device) = with_book().await;
    let other = harness.link("Other Kobo", common::API_KEY).await;
    harness
        .report_state(&other, BOOK, update("Reading", Some("kobo.2.1")))
        .await;

    assert_ne!(fetch(&harness, &device).await["LastModified"], EPOCH);
}

#[tokio::test]
async fn refuses_an_update_carrying_no_reading_state() {
    let (harness, device) = with_book().await;

    let response = harness
        .json(
            Method::PUT,
            &device.at("/v1/library/book/state"),
            json!({ "ReadingStates": [] }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn answers_not_found_for_a_book_prosa_does_not_hold() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let fetched = harness.get(&device.at("/v1/library/book/state")).await;
    let updated = harness
        .json(
            Method::PUT,
            &device.at("/v1/library/book/state"),
            update("Reading", None),
        )
        .await;

    assert_eq!(fetched.status(), StatusCode::NOT_FOUND);
    assert_eq!(updated.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn keeps_the_rating_the_device_gives_and_hands_it_back() {
    let (harness, device) = with_book().await;

    let rated = harness
        .request(Method::POST, &device.at("/v1/products/book/rating/4"), &[])
        .await;
    let reviews = body_json(harness.get(&device.at("/v1/user/reviews?ProductIds=book")).await).await;

    assert_eq!(rated.status(), StatusCode::OK);
    assert_eq!(reviews["Items"][0]["Rating"], 4);
    assert_eq!(reviews["Items"][0]["ProductId"], BOOK);
}

#[tokio::test]
async fn clears_the_rating_when_the_device_rates_zero() {
    let (harness, device) = with_book().await;
    harness
        .request(Method::POST, &device.at("/v1/products/book/rating/4"), &[])
        .await;

    harness
        .request(Method::POST, &device.at("/v1/products/book/rating/0"), &[])
        .await;
    let reviews = body_json(harness.get(&device.at("/v1/user/reviews?ProductIds=book")).await).await;

    assert_eq!(reviews["Items"], json!([]));
}

#[tokio::test]
async fn refuses_to_look_up_ratings_without_naming_a_book() {
    let (harness, device) = with_book().await;

    let response = harness.get(&device.at("/v1/user/reviews")).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn passes_on_prosa_refusing_the_key() {
    let (harness, device) = with_book().await;
    harness
        .client
        .fail(ProsaMethod::FetchState, ClientError::Forbidden);

    let response = harness.get(&device.at("/v1/library/book/state")).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
