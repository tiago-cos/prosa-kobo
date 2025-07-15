use crate::app::covers::models::{CoverToken, CoverTokenError};
use sqlx::SqlitePool;

pub async fn add_token(pool: &SqlitePool, book_id: &str, token: &str, api_key: &str, expiration: i64) -> () {
    sqlx::query(
        r#"
        INSERT INTO cover_tokens (book_id, token, api_key, expiration)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(book_id)
    .bind(token)
    .bind(api_key)
    .bind(expiration)
    .execute(pool)
    .await
    .expect("Failed to add cover token");
}

pub async fn get_token(pool: &SqlitePool, token: &str) -> Result<CoverToken, CoverTokenError> {
    let token: CoverToken = sqlx::query_as(
        r#"
        SELECT book_id, expiration, api_key
        FROM cover_tokens
        WHERE token = $1
        "#,
    )
    .bind(token)
    .fetch_one(pool)
    .await?;

    Ok(token)
}

pub async fn delete_token(pool: &SqlitePool, token: &str) -> () {
    sqlx::query(
        r#"
        DELETE FROM cover_tokens
        WHERE token = $1
        "#,
    )
    .bind(token)
    .execute(pool)
    .await
    .expect("Failed to delete cover token");
}
