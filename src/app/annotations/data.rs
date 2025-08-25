use sqlx::SqlitePool;

pub async fn get_etag(pool: &SqlitePool, book_id: &str) -> Option<String> {
    sqlx::query_scalar(
        r"
        SELECT etag
        FROM etags
        WHERE book_id = $1
        ",
    )
    .bind(book_id)
    .fetch_optional(pool)
    .await
    .expect("Failed to get etag")
}

pub async fn update_etag(pool: &SqlitePool, book_id: &str, etag: &str) -> () {
    sqlx::query(
        r"
        INSERT OR REPLACE INTO etags (book_id, etag)
        VALUES ($1, $2)
        ",
    )
    .bind(book_id)
    .bind(etag)
    .execute(pool)
    .await
    .expect("Failed to replace etag");
}

pub async fn delete_etag(pool: &SqlitePool, book_id: &str) -> () {
    sqlx::query(
        r"
        DELETE FROM etags
        WHERE book_id = $1
        ",
    )
    .bind(book_id)
    .execute(pool)
    .await
    .expect("Failed to delete etag");
}
