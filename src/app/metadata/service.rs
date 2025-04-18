use super::{BookMetadata, DownloadUrl};
use crate::{app::books, client::prosa::Client};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use urlencoding::encode;

pub async fn translate_metadata(
    pool: &SqlitePool,
    client: &Client,
    book_id: &str,
    server_url: &str,
    download_expiration: i64,
    api_key: &str,
) -> BookMetadata {
    let size_response = client
        .fetch_size(&book_id, api_key)
        .expect("Size response should not fail");
    let metadata_response = client
        .fetch_metadata(&book_id, api_key)
        .expect("Metadata response should not fail");
    let download_token = books::generate_token(pool, book_id, download_expiration, api_key).await;
    let download_token = encode(&download_token).to_string();
    let download_url = format!("{}/books/{}?token={}", server_url, book_id, download_token);

    let download_url = DownloadUrl::new(&download_url, size_response);
    let metadata = BookMetadata::new(&book_id, metadata_response, download_url);

    metadata
}

pub fn unix_millis_to_string(timestamp_millis: i64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp_millis(timestamp_millis)
        .expect("Failed to convert timesstamp to string");

    let formatted = format!(
        "{}.{:07}Z",
        datetime.format("%Y-%m-%dT%H:%M:%S"),
        datetime.timestamp_subsec_nanos() / 100
    );

    formatted
}
