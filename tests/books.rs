mod common;

use axum::http::{Method, StatusCode};
use common::{DEVICE_HARDWARE_ID, Harness, body_bytes, body_json, fixture};
use prosa_kobo::client::{mock::ProsaMethod, prosa::ClientError};
use std::io::{Cursor, Read};

const BOOK: &str = "book";

fn is_kepub(bytes: &[u8]) -> bool {
    let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(bytes)) else {
        return false;
    };

    let documents: Vec<String> = archive
        .file_names()
        .filter(|name| name.contains(".htm"))
        .map(str::to_owned)
        .collect();

    documents.iter().any(|name| {
        let mut body = String::new();
        archive
            .by_name(name)
            .is_ok_and(|mut file| file.read_to_string(&mut body).is_ok())
            && body.contains("koboSpan")
    })
}

#[tokio::test]
async fn hands_the_device_a_kepub_rather_than_the_epub_prosa_stores() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    let epub = fixture("The_Great_Gatsby.epub");
    harness.client.seed_file(BOOK, epub.clone());

    let response = harness.get(&device.at("/books/book")).await;

    assert_eq!(response.status(), StatusCode::OK);

    let kepub = body_bytes(response).await;

    assert!(is_kepub(&kepub));
    assert_ne!(kepub, epub);
}

#[tokio::test]
async fn delivers_exactly_the_size_metadata_advertised() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_file(BOOK, fixture("The_Great_Gatsby.epub"));

    let metadata = body_json(harness.get(&device.at("/v1/library/book/metadata")).await).await;
    let advertised = metadata[0]["DownloadUrls"][0]["Size"]
        .as_u64()
        .expect("Size should be a number");

    let downloaded = body_bytes(harness.get(&device.at("/books/book")).await).await;

    assert_eq!(advertised, downloaded.len() as u64);
}

#[tokio::test]
async fn converts_a_book_once_however_often_it_is_asked_for() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_file(BOOK, fixture("The_Great_Gatsby.epub"));

    let first = body_bytes(harness.get(&device.at("/books/book")).await).await;
    let second = body_bytes(harness.get(&device.at("/books/book")).await).await;

    assert_eq!(first, second);
    assert_eq!(harness.client.call_count(ProsaMethod::DownloadBook), 1);
}

#[tokio::test]
async fn answers_not_found_for_a_book_prosa_does_not_hold() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness.get(&device.at("/books/book")).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn passes_on_prosa_refusing_the_key() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_file(BOOK, fixture("The_Great_Gatsby.epub"));
    harness
        .client
        .fail(ProsaMethod::DownloadBook, ClientError::Forbidden);

    let response = harness.get(&device.at("/books/book")).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn fails_rather_than_serving_a_file_that_is_not_an_epub() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_file(BOOK, fixture("This_is_not_an_epub.txt"));

    let response = harness.get(&device.at("/books/book")).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn deletes_the_book_from_prosa() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_book(BOOK);

    let response = harness
        .request(Method::DELETE, &device.at("/v1/library/book"), &[])
        .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(harness.client.call_count(ProsaMethod::DeleteBook), 1);
    assert_eq!(
        harness.get(&device.at("/books/book")).await.status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn deleting_a_book_prosa_no_longer_holds_still_succeeds() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness
        .request(Method::DELETE, &device.at("/v1/library/book"), &[])
        .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn deleting_a_book_forgets_its_conversion() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_file(BOOK, fixture("The_Great_Gatsby.epub"));

    harness.get(&device.at("/books/book")).await;
    harness
        .request(Method::DELETE, &device.at("/v1/library/book"), &[])
        .await;

    harness
        .client
        .seed_file(BOOK, fixture("The_Wonderful_Wizard_of_Oz.epub"));
    harness.get(&device.at("/books/book")).await;

    assert_eq!(harness.client.call_count(ProsaMethod::DownloadBook), 2);
}

#[tokio::test]
async fn deleting_a_book_forgets_its_annotation_etag() {
    let harness = Harness::new().await;
    let device = harness.introduced().await;
    harness.client.seed_book(BOOK);
    let hardware = [("x-kobo-deviceid", DEVICE_HARDWARE_ID)];
    let etag = |response: axum::http::Response<axum::body::Body>| {
        response.headers()["ETag"]
            .to_str()
            .expect("The ETag should be text")
            .to_owned()
    };

    let before = etag(
        harness
            .request(Method::GET, "/api/v3/content/book/annotations", &hardware)
            .await,
    );
    harness
        .request(Method::DELETE, &device.at("/v1/library/book"), &[])
        .await;
    harness.client.seed_book(BOOK);
    let after = etag(
        harness
            .request(Method::GET, "/api/v3/content/book/annotations", &hardware)
            .await,
    );

    assert_ne!(before, after);
}
