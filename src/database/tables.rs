use sqlx::SqlitePool;

pub async fn create_tables(pool: &SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS linked_devices (
            device_id TEXT PRIMARY KEY NOT NULL,
            api_key TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS unlinked_devices (
            device_id TEXT PRIMARY KEY NOT NULL,
            timestamp BIGINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS book_tokens (
            book_id TEXT NOT NULL,
            token TEXT NOT NULL,
            api_key TEXT NOT NULL,
            expiration BIGINT NOT NULL,
            PRIMARY KEY(book_id, token)
        );

        CREATE TABLE IF NOT EXISTS cover_tokens (
            book_id TEXT NOT NULL,
            token TEXT NOT NULL,
            api_key TEXT NOT NULL,
            expiration BIGINT NOT NULL,
            PRIMARY KEY(book_id, token)
        );

        CREATE TABLE IF NOT EXISTS etags (
            book_id TEXT PRIMARY KEY NOT NULL,
            etag TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create tables");
}

pub async fn clear_tables(pool: &SqlitePool) {
    sqlx::query(
        r#"
        DROP TABLE IF EXISTS book_tokens;
        DROP TABLE IF EXISTS cover_tokens;
        DROP TABLE IF EXISTS linked_devices;
        DROP TABLE IF EXISTS unlinked_devices;
        DROP TABLE IF EXISTS etags;
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to drop tables");
}
