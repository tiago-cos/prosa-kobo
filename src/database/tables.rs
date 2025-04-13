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
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create device tables");
}

pub async fn clear_tables(pool: &SqlitePool) {
    sqlx::query(
        r#"
        DROP TABLE IF EXISTS linked_devices;
        DROP TABLE IF EXISTS unlinked_devices;
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to drop device tables");
}
