use super::{BookMetadata, DownloadUrl};
use crate::{
    app::{covers, error::KoboError},
    client::{
        ProsaMetadata,
        prosa::{ClientError, ProsaApi},
    },
};
use sqlx::SqlitePool;

pub async fn translate_metadata(
    pool: &SqlitePool,
    client: &dyn ProsaApi,
    book_id: &str,
    server_url: &str,
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

    let download_url = format!("{server_url}/books/{book_id}");
    let download_url = DownloadUrl::new(&download_url, size_response);

    let cover_token = covers::get_token(pool, book_id, device_id).await;

    metadata.download_urls.push(download_url);
    metadata.cover_image_id = format!("{book_id}?v={cover_token}");

    Ok(metadata)
}
