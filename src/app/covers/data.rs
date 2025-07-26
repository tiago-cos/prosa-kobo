use crate::app::covers::models::{CoverToken, CoverTokenError};
use sqlx::SqlitePool;

pub async fn add_token(pool: &SqlitePool, book_id: &str, token: &str, api_key: &str) -> () {
    sqlx::query(
        r#"
        INSERT INTO cover_tokens (book_id, token, api_key)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(book_id)
    .bind(token)
    .bind(api_key)
    .execute(pool)
    .await
    .expect("Failed to add cover token");
}

pub async fn get_token(pool: &SqlitePool, token: &str) -> Result<CoverToken, CoverTokenError> {
    let token: CoverToken = sqlx::query_as(
        r#"
        SELECT book_id, api_key
        FROM cover_tokens
        WHERE token = $1
        "#,
    )
    .bind(token)
    .fetch_one(pool)
    .await?;

    Ok(token)
}
