mod common;

use axum::http::StatusCode;
use common::{Harness, body_json};
use prosa_kobo::client::{
    ProsaMetadata,
    metadata::{ProsaContributor, ProsaSeries},
    mock::ProsaMethod,
    prosa::ClientError,
};
use serde_json::Value;

const BOOK: &str = "book";

fn gatsby() -> ProsaMetadata {
    ProsaMetadata {
        title: Some("The Great Gatsby".to_owned()),
        subtitle: Some("A novel".to_owned()),
        description: Some("A story of the Jazz Age.".to_owned()),
        publisher: Some("Scribner".to_owned()),
        publication_date: Some(-1_411_462_448_000),
        isbn: Some("978-0-7432-7356-5".to_owned()),
        contributors: Some(vec![ProsaContributor {
            name: "F. Scott Fitzgerald".to_owned(),
            role: "Author".to_owned(),
        }]),
        genres: Some(vec!["Fiction".to_owned()]),
        series: Some(ProsaSeries {
            title: "Classics".to_owned(),
            number: Some(3.0),
        }),
        page_count: Some(208),
        language: Some("English".to_owned()),
    }
}

async fn metadata(harness: &Harness, path: &str) -> Value {
    let response = harness.get(path).await;
    assert_eq!(response.status(), StatusCode::OK);

    body_json(response).await[0].clone()
}

#[tokio::test]
async fn carries_what_prosa_knows_about_the_book() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book(BOOK);
    harness.client.seed_metadata(BOOK, gatsby());

    let metadata = metadata(&harness, &device.at("/v1/library/book/metadata")).await;

    assert_eq!(metadata["Title"], "The Great Gatsby");
    assert_eq!(metadata["Subtitle"], "A novel");
    assert_eq!(metadata["Description"], "A story of the Jazz Age.");
    assert_eq!(metadata["Publisher"]["Name"], "Scribner");
    assert_eq!(metadata["Isbn"], "978-0-7432-7356-5");
    assert_eq!(metadata["Contributors"][0], "F. Scott Fitzgerald");
    assert_eq!(metadata["ContributorRoles"][0]["Role"], "Author");
    assert_eq!(metadata["Series"]["Name"], "Classics");
    assert_eq!(metadata["Series"]["Number"], "3");
    assert_eq!(metadata["EntitlementId"], BOOK);
}

#[tokio::test]
async fn names_the_language_by_its_three_letter_code() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book(BOOK);
    harness.client.seed_metadata(BOOK, gatsby());

    let metadata = metadata(&harness, &device.at("/v1/library/book/metadata")).await;

    assert_eq!(metadata["Language"], "eng");
}

#[tokio::test]
async fn writes_the_publication_date_the_way_the_device_reads_dates() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book(BOOK);
    harness.client.seed_metadata(BOOK, gatsby());

    let metadata = metadata(&harness, &device.at("/v1/library/book/metadata")).await;

    assert_eq!(metadata["PublicationDate"], "1925-04-10T15:05:52.0000000Z");
}

#[tokio::test]
async fn answers_for_a_book_prosa_holds_no_metadata_for() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book(BOOK);

    let metadata = metadata(&harness, &device.at("/v1/library/book/metadata")).await;

    assert_eq!(metadata["EntitlementId"], BOOK);
    assert_eq!(metadata["Title"], "Untitled");
}

#[tokio::test]
async fn names_a_book_whose_title_is_blank_untitled() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book(BOOK);
    harness.client.seed_metadata(
        BOOK,
        ProsaMetadata {
            title: Some("  ".to_owned()),
            ..gatsby()
        },
    );

    let metadata = metadata(&harness, &device.at("/v1/library/book/metadata")).await;

    assert_eq!(metadata["Title"], "Untitled");
    assert_eq!(metadata["Subtitle"], "A novel");
}

#[tokio::test]
async fn points_the_download_back_under_the_device_as_a_kepub() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book(BOOK);

    let metadata = metadata(&harness, &device.at("/v1/library/book/metadata")).await;
    let download = &metadata["DownloadUrls"][0];

    assert_eq!(download["Format"], "KEPUB");
    assert!(
        download["Url"]
            .as_str()
            .expect("Url should be a string")
            .ends_with(&device.at("/books/book"))
    );
}

#[tokio::test]
async fn answers_not_found_for_a_book_prosa_does_not_hold() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness.get(&device.at("/v1/library/book/metadata")).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn passes_on_prosa_refusing_the_key() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.add_book(BOOK);
    harness
        .client
        .fail(ProsaMethod::FetchMetadata, ClientError::Forbidden);

    let response = harness.get(&device.at("/v1/library/book/metadata")).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
