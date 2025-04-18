use sqlx::SqlitePool;

use super::models::{DownloadToken, TokenError};

pub async fn add_token(pool: &SqlitePool, token: &str, expiration: i64) -> () {
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO download_tokens (token, expiration)
        VALUES ($1, $2)
        "#,
    )
    .bind(token)
    .bind(expiration)
    .execute(pool)
    .await
    .expect("Failed to add download token");
}

pub async fn get_token(pool: &SqlitePool, token: &str) -> Result<DownloadToken, TokenError> {
    let token: DownloadToken = sqlx::query_as(
        r#"
        SELECT token, expiration
        FROM download_tokens
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
        DELETE FROM download_tokens
        WHERE token = $1
        "#,
    )
    .bind(token)
    .execute(pool)
    .await
    .expect("Failed to delete download token");
}
