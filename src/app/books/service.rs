use super::data;
use crate::{
    app::{
        books::models::{BookTokenError, BOOK_TOKEN_SIZE},
        error::KoboError,
    },
    client::prosa::Client,
};
use base64::{prelude::BASE64_URL_SAFE, Engine};
use rand::RngCore;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn download_book(
    pool: &SqlitePool,
    client: &Client,
    book_id: &str,
    book_token: &str,
) -> Result<Vec<u8>, KoboError> {
    let api_key = verify_token(pool, book_id, book_token).await?;
    let book = client.download_book(&book_id, &api_key)?;
    Ok(book)
}

pub async fn delete_book(client: &Client, book_id: &str, api_key: &str) -> Result<(), KoboError> {
    client.delete_book(&book_id, &api_key)?;
    Ok(())
}

pub async fn generate_token(pool: &SqlitePool, book_id: &str, expiration: i64, api_key: &str) -> String {
    let mut bytes = vec![0u8; BOOK_TOKEN_SIZE];
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

async fn verify_token(pool: &SqlitePool, book_id: &str, token: &str) -> Result<String, BookTokenError> {
    let verifier = data::get_token(&pool, token).await?;

    let now: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs()
        .try_into()
        .expect("Failed to convert timestamp");

    if now > verifier.expiration {
        data::delete_token(&pool, token).await;
        return Err(BookTokenError::InvalidToken);
    }

    if !(verifier.book_id == book_id) {
        return Err(BookTokenError::InvalidToken);
    }

    data::delete_token(&pool, token).await;

    Ok(verifier.api_key)
}
