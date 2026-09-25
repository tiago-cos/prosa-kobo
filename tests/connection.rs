use prosa_kobo::database::connect;
use sqlx::SqlitePool;

async fn pragma<T>(pool: &SqlitePool, name: &str) -> T
where
    T: for<'r> sqlx::Decode<'r, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Send + Unpin,
{
    sqlx::query_scalar(&format!("PRAGMA {name}"))
        .fetch_one(pool)
        .await
        .expect("Failed to read a pragma")
}

#[tokio::test]
async fn connects_in_wal_mode_waiting_out_contended_writes() {
    let file = std::env::temp_dir().join(format!("prosa-kobo-connection-{:016x}.db", rand::random::<u64>()));
    let pool = connect(&file.to_string_lossy()).await.expect("Failed to connect");

    assert_eq!(pragma::<String>(&pool, "journal_mode").await, "wal");
    assert_eq!(pragma::<i64>(&pool, "synchronous").await, 1);
    assert_eq!(pragma::<i64>(&pool, "foreign_keys").await, 1);
    assert_eq!(pragma::<i64>(&pool, "busy_timeout").await, 10_000);

    pool.close().await;
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", file.to_string_lossy()));
    }
}
