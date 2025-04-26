use super::data;
use crate::{
    app::{
        error::KoboError,
        tokens::{self, TokenError},
    },
    client::prosa::Client,
};
use sqlx::SqlitePool;

pub async fn download_book(
    pool: &SqlitePool,
    client: &Client,
    book_id: &str,
    token: &str,
) -> Result<Vec<u8>, KoboError> {
    let api_key = verify_token(pool, book_id, token).await?;
    let book = client.download_book(&book_id, &api_key)?;
    Ok(book)
}

pub async fn generate_token(pool: &SqlitePool, book_id: &str, expiration: i64, api_key: &str) -> String {
    let token = tokens::generate_token(pool, expiration).await;
    data::add_token(pool, book_id, &token, api_key).await;

    token
}

async fn verify_token(pool: &SqlitePool, book_id: &str, token: &str) -> Result<String, TokenError> {
    let (book_id_verifier, api_key) = data::get_token(pool, token).await?;

    if !(book_id_verifier == book_id) {
        return Err(TokenError::InvalidToken);
    }

    tokens::verify_token(pool, token).await?;

    Ok(api_key)
}

pub async fn delete_book(client: &Client, book_id: &str, api_key: &str) -> Result<(), KoboError> {
    client.delete_book(&book_id, &api_key)?;
    Ok(())
}
