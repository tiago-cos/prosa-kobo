use super::{BookMetadata, DownloadUrl};
use crate::{
    app::{covers, error::KoboError, kepub},
    client::{
        ProsaMetadata,
        prosa::{ClientError, ProsaApi},
    },
};

pub async fn translate_metadata(
    client: &dyn ProsaApi,
    book_id: &str,
    server_url: &str,
    api_key: &str,
    device_id: &str,
) -> Result<BookMetadata, KoboError> {
    let kepub_size = kepub::get_kepub(client, book_id, api_key).await?.len() as u64;
    let metadata_response = match client.fetch_metadata(book_id, api_key) {
        Ok(response) => response,
        Err(ClientError::NotFound) => ProsaMetadata::default(),
        Err(e) => return Err(e.into()),
    };

    let mut metadata = BookMetadata::new(book_id, metadata_response);

    let download_url = format!("{server_url}/books/{book_id}");
    let download_url = DownloadUrl::new(&download_url, kepub_size);

    let cover_version = covers::get_version(device_id, book_id).await;

    metadata.download_urls.push(download_url);
    metadata.cover_image_id = format!("{book_id}?v={cover_version}");

    Ok(metadata)
}
