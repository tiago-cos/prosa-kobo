use crate::app::tokens::TokenError;
use sqlx::SqlitePool;

pub async fn add_token(pool: &SqlitePool, book_id: &str, token: &str, api_key: &str) -> () {
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO file_tokens (book_id, token, api_key)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(book_id)
    .bind(token)
    .bind(api_key)
    .execute(pool)
    .await
    .expect("Failed to add book token");
}

pub async fn get_token(pool: &SqlitePool, token: &str) -> Result<(String, String), TokenError> {
    let (book_id, api_key): (String, String) = sqlx::query_as(
        r#"
        SELECT book_id, api_key
        FROM file_tokens
        WHERE token = $1
        "#,
    )
    .bind(token)
    .fetch_one(pool)
    .await?;

    Ok((book_id, api_key))
}
