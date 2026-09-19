use sqlx::SqlitePool;

pub async fn create_tables(pool: &SqlitePool) {
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS linked_devices (
            device_id TEXT PRIMARY KEY NOT NULL,
            lookup_key TEXT NOT NULL UNIQUE,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            api_key TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS cover_versions (
            device_id TEXT NOT NULL,
            book_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            PRIMARY KEY(device_id, book_id)
        );

        CREATE TABLE IF NOT EXISTS etags (
            book_id TEXT PRIMARY KEY NOT NULL,
            etag TEXT NOT NULL
        );
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create tables");
}

pub async fn clear_tables(pool: &SqlitePool) {
    sqlx::query(
        r"
        DROP TABLE IF EXISTS cover_versions;
        DROP TABLE IF EXISTS linked_devices;
        DROP TABLE IF EXISTS etags;
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to drop tables");
}
