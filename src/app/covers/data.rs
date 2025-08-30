use crate::app::covers::models::{CoverToken, CoverTokenError};
use sqlx::SqlitePool;

pub async fn add_token(pool: &SqlitePool, book_id: &str, token: &str, device_id: &str) -> () {
    sqlx::query(
        r"
        INSERT INTO cover_tokens (book_id, token, device_id)
        VALUES ($1, $2, $3)
        ",
    )
    .bind(book_id)
    .bind(token)
    .bind(device_id)
    .execute(pool)
    .await
    .expect("Failed to add cover token");
}

pub async fn verify_token(pool: &SqlitePool, token: &str) -> Result<CoverToken, CoverTokenError> {
    let token: CoverToken = sqlx::query_as(
        r"
        SELECT book_id, device_id
        FROM cover_tokens
        WHERE token = $1
        ",
    )
    .bind(token)
    .fetch_one(pool)
    .await?;

    Ok(token)
}

pub async fn get_token(pool: &SqlitePool, device_id: &str, book_id: &str) -> Option<String> {
    sqlx::query_scalar(
        r"
        SELECT token
        FROM cover_tokens
        WHERE device_id = $1 AND book_id = $2
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .fetch_optional(pool)
    .await
    .expect("Failed to fetch cover token")
}

pub async fn delete_token(pool: &SqlitePool, book_id: &str, device_id: &str) -> () {
    sqlx::query(
        r"
        DELETE FROM cover_tokens
        WHERE device_id = $1 AND book_id = $2
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .execute(pool)
    .await
    .expect("Failed to delete cover token");
}
