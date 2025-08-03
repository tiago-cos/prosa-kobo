use crate::app::books::models::{BookToken, BookTokenError};
use sqlx::SqlitePool;

pub async fn add_token(
    pool: &SqlitePool,
    book_id: &str,
    token: &str,
    device_id: &str,
    expiration: i64,
) -> () {
    sqlx::query(
        r#"
        INSERT INTO book_tokens (book_id, token, device_id, expiration)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(book_id)
    .bind(token)
    .bind(device_id)
    .bind(expiration)
    .execute(pool)
    .await
    .expect("Failed to add book token");
}

pub async fn get_token(pool: &SqlitePool, token: &str) -> Result<BookToken, BookTokenError> {
    let token: BookToken = sqlx::query_as(
        r#"
        SELECT book_id, expiration, device_id
        FROM book_tokens
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
        DELETE FROM book_tokens
        WHERE token = $1
        "#,
    )
    .bind(token)
    .execute(pool)
    .await
    .expect("Failed to delete download token");
}

pub async fn delete_book_tokens(pool: &SqlitePool, book_id: &str) -> () {
    sqlx::query(
        r#"
        DELETE FROM book_tokens
        WHERE book_id = $1
        "#,
    )
    .bind(book_id)
    .execute(pool)
    .await
    .expect("Failed to delete download book tokens");
}
