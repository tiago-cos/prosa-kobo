use crate::{
    app::{
        covers::{
            data,
            models::{CoverTokenError, COVER_TOKEN_SIZE},
        },
        error::KoboError,
    },
    client::prosa::Client,
};
use base64::{prelude::BASE64_URL_SAFE, Engine};
use rand::RngCore;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

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

pub async fn generate_token(pool: &SqlitePool, book_id: &str, expiration: i64, api_key: &str) -> String {
    let mut bytes = vec![0u8; COVER_TOKEN_SIZE * 8];
    rand::rng().fill_bytes(&mut bytes);
    let token = BASE64_URL_SAFE.encode(bytes);

    let now: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs()
        .try_into()
        .expect("Failed to convert timestamp");

    let expiration = now + expiration;

    data::add_token(pool, book_id, &token, api_key, expiration).await;

    token
}

async fn verify_token(pool: &SqlitePool, book_id: &str, token: &str) -> Result<String, CoverTokenError> {
    let verifier = data::get_token(&pool, token).await?;

    let now: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs()
        .try_into()
        .expect("Failed to convert timestamp");

    if now > verifier.expiration {
        data::delete_token(&pool, token).await;
        return Err(CoverTokenError::InvalidToken);
    }

    if !(verifier.book_id == book_id) {
        return Err(CoverTokenError::InvalidToken);
    }

    Ok(verifier.api_key)
}
