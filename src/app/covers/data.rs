use sqlx::SqlitePool;

pub async fn get_version(pool: &SqlitePool, device_id: &str, book_id: &str) -> Option<i64> {
    sqlx::query_scalar(
        r"
        SELECT version
        FROM cover_versions
        WHERE device_id = $1 AND book_id = $2
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .fetch_optional(pool)
    .await
    .expect("Failed to get cover version")
}

pub async fn bump_version(pool: &SqlitePool, device_id: &str, book_id: &str) {
    sqlx::query(
        r"
        INSERT INTO cover_versions (device_id, book_id, version)
        VALUES ($1, $2, 1)
        ON CONFLICT(device_id, book_id) DO UPDATE SET version = version + 1
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .execute(pool)
    .await
    .expect("Failed to bump cover version");
}
