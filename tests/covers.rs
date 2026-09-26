mod common;

use axum::http::StatusCode;
use common::{Harness, body_bytes};
use image::{GenericImageView, ImageFormat, RgbImage};
use prosa_kobo::client::{mock::ProsaMethod, prosa::ClientError};
use std::io::Cursor;

const BOOK: &str = "book";

fn cover(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());

    RgbImage::new(width, height)
        .write_to(&mut bytes, ImageFormat::Jpeg)
        .expect("Failed to encode the cover");

    bytes.into_inner()
}

#[tokio::test]
async fn serves_the_cover_prosa_holds() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    let original = cover(60, 90);
    harness.client.seed_cover(BOOK, original.clone());

    let response = harness.get(&device.at("/images/book")).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_bytes(response).await, original);
}

#[tokio::test]
async fn resizes_the_cover_to_the_size_the_device_asks_for() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_cover(BOOK, cover(60, 90));

    let response = harness.get(&device.at("/images/book?width=20&height=30")).await;

    assert_eq!(response.status(), StatusCode::OK);

    let resized =
        image::load_from_memory(&body_bytes(response).await).expect("The resized cover should be an image");

    assert_eq!(resized.dimensions(), (20, 30));
}

#[tokio::test]
async fn resizes_the_cover_named_as_the_device_builds_its_url() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_cover(BOOK, cover(60, 90));

    let response = harness
        .get(&device.at("/images/book?v=3&width=20&height=30"))
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let resized =
        image::load_from_memory(&body_bytes(response).await).expect("The resized cover should be an image");

    assert_eq!(resized.dimensions(), (20, 30));
}

#[tokio::test]
async fn leaves_the_cover_alone_when_only_one_dimension_is_given() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    let original = cover(60, 90);
    harness.client.seed_cover(BOOK, original.clone());

    let response = harness.get(&device.at("/images/book?width=20")).await;

    assert_eq!(body_bytes(response).await, original);
}

#[tokio::test]
async fn serves_a_cover_it_cannot_resize_as_it_is() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    let unreadable = b"not an image".to_vec();
    harness.client.seed_cover(BOOK, unreadable.clone());

    let response = harness.get(&device.at("/images/book?width=20&height=30")).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_bytes(response).await, unreadable);
}

#[tokio::test]
async fn answers_not_found_for_a_book_without_a_cover() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_book(BOOK);

    let response = harness.get(&device.at("/images/book")).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn answers_not_found_for_a_book_prosa_does_not_hold() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness.get(&device.at("/images/book")).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn passes_on_prosa_refusing_the_key() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_cover(BOOK, cover(60, 90));
    harness
        .client
        .fail(ProsaMethod::DownloadCover, ClientError::Forbidden);

    let response = harness.get(&device.at("/images/book")).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
