use crate::{
    app::{
        covers::{
            data,
            models::{CoverTokenError, COVER_TOKEN_SIZE},
        },
        devices,
        error::KoboError,
    },
    client::prosa::Client,
};
use base64::{prelude::BASE64_URL_SAFE, Engine};
use rand::RngCore;
use sqlx::SqlitePool;

pub async fn download_cover(
    pool: &SqlitePool,
    client: &Client,
    book_id: &str,
    cover_token: &str,
) -> Result<Vec<u8>, KoboError> {
    let api_key = verify_token(pool, book_id, cover_token).await?;
    let cover = client.download_cover(&book_id, &api_key)?;
    Ok(cover)
}

pub async fn generate_token(pool: &SqlitePool, book_id: &str, device_id: &str) -> String {
    let mut bytes = vec![0u8; COVER_TOKEN_SIZE];
    rand::rng().fill_bytes(&mut bytes);
    let token = BASE64_URL_SAFE.encode(bytes);

    data::delete_token(pool, book_id).await;
    data::add_token(pool, book_id, &token, device_id).await;

    token
}

async fn verify_token(pool: &SqlitePool, book_id: &str, token: &str) -> Result<String, CoverTokenError> {
    let verifier = data::get_token(&pool, token).await?;

    if !(verifier.book_id == book_id) {
        return Err(CoverTokenError::InvalidToken);
    }

    let api_key = match devices::service::get_linked_device(pool, &verifier.device_id).await {
        Some(d) => d.api_key,
        None => return Err(CoverTokenError::InvalidToken),
    };

    Ok(api_key)
}
