use super::{BookMetadata, DownloadUrl};
use crate::{
    app::{books, error::KoboError},
    client::{
        prosa::{Client, ClientError},
        ProsaMetadata,
    },
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sqlx::SqlitePool;
use urlencoding::encode;

pub async fn translate_metadata(
    pool: &SqlitePool,
    client: &Client,
    book_id: &str,
    server_url: &str,
    download_expiration: i64,
    api_key: &str,
) -> Result<BookMetadata, KoboError> {
    let size_response = client.fetch_size(&book_id, api_key)?;
    let metadata_response = match client.fetch_metadata(&book_id, api_key) {
        Ok(response) => response,
        Err(ClientError::NotFound) => ProsaMetadata::default(),
        Err(e) => return Err(e.into()),
    };

    let download_token = books::generate_token(pool, book_id, download_expiration, api_key).await;
    let download_token = encode(&download_token).to_string();
    let download_url = format!("{}/books/{}?token={}", server_url, book_id, download_token);

    let download_url = DownloadUrl::new(&download_url, size_response);
    let metadata = BookMetadata::new(&book_id, metadata_response, download_url);

    Ok(metadata)
}

pub fn random_string(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    rand::rng().fill_bytes(&mut bytes);
    let mut random = BASE64_URL_SAFE_NO_PAD.encode(bytes);
    random.truncate(len);
    random
}
