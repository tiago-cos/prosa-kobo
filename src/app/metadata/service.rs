use super::{BookMetadata, DownloadUrl};
use crate::{
    app::{books, covers, error::KoboError},
    client::{
        ProsaMetadata,
        prosa::{Client, ClientError},
    },
};
use sqlx::SqlitePool;

pub async fn translate_metadata(
    pool: &SqlitePool,
    client: &Client,
    book_id: &str,
    server_url: &str,
    book_expiration: i64,
    api_key: &str,
    device_id: &str,
) -> Result<BookMetadata, KoboError> {
    let size_response = client.fetch_book_file_metadata(book_id, api_key)?.file_size;
    let metadata_response = match client.fetch_metadata(book_id, api_key) {
        Ok(response) => response,
        Err(ClientError::NotFound) => ProsaMetadata::default(),
        Err(e) => return Err(e.into()),
    };

    let mut metadata = BookMetadata::new(book_id, metadata_response);

    let book_token = books::generate_token(pool, book_id, book_expiration, device_id).await;
    let download_url = format!("{server_url}/books/{book_id}?token={book_token}");
    let download_url = DownloadUrl::new(&download_url, size_response);

    let cover_token = covers::generate_token(pool, book_id, device_id).await;
    let cover_token = format!("?token={cover_token}");

    metadata.download_urls.push(download_url);
    metadata.cover_image_id.push_str(&cover_token);

    Ok(metadata)
}
