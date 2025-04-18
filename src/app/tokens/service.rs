use super::{
    data,
    models::{TokenError, TOKEN_SIZE},
};
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::RngCore;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn generate_token(pool: &SqlitePool, expiration: i64) -> String {
    let mut bytes = vec![0u8; TOKEN_SIZE];
    rand::rng().fill_bytes(&mut bytes);
    let token = BASE64_STANDARD.encode(bytes);

    let now: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs()
        .try_into()
        .expect("Failed to convert timestamp");

    let expiration = now + expiration;

    data::add_token(&pool, &token, expiration).await;

    token
}

pub async fn verify_token(pool: &SqlitePool, token: &str) -> Result<(), TokenError> {
    let verifier = data::get_token(&pool, token).await?;

    let now: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs()
        .try_into()
        .expect("Failed to convert timestamp");

    if !(verifier.token == token && now <= verifier.expiration) {
        data::delete_token(&pool, token).await;
        return Err(TokenError::InvalidToken);
    }

    data::delete_token(&pool, token).await;

    Ok(())
}
