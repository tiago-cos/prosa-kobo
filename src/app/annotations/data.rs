use sqlx::SqliteExecutor;

pub async fn get_etag<'e>(db: impl SqliteExecutor<'e>, book_id: &str) -> Option<String> {
    sqlx::query_scalar(
        r"
        SELECT etag
        FROM etags
        WHERE book_id = $1
        ",
    )
    .bind(book_id)
    .fetch_optional(db)
    .await
    .expect("Failed to get etag")
}

pub async fn update_etag<'e>(db: impl SqliteExecutor<'e>, book_id: &str, etag: &str) -> () {
    sqlx::query(
        r"
        INSERT OR REPLACE INTO etags (book_id, etag)
        VALUES ($1, $2)
        ",
    )
    .bind(book_id)
    .bind(etag)
    .execute(db)
    .await
    .expect("Failed to replace etag");
}

pub async fn delete_etag<'e>(db: impl SqliteExecutor<'e>, book_id: &str) -> () {
    sqlx::query(
        r"
        DELETE FROM etags
        WHERE book_id = $1
        ",
    )
    .bind(book_id)
    .execute(db)
    .await
    .expect("Failed to delete etag");
}
