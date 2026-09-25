use crate::CONFIG;
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::time::Duration;

pub type DatabaseError = Box<dyn std::error::Error + Send + Sync>;

fn connect_options(filename: &str) -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .filename(filename)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(CONFIG.database.busy_timeout_seconds))
}

pub async fn connect(filename: &str) -> Result<Pool<Sqlite>, DatabaseError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(CONFIG.database.max_connections)
        .connect_with(connect_options(filename))
        .await?;

    Ok(pool)
}

pub async fn init(filename: &str) -> Result<Pool<Sqlite>, DatabaseError> {
    let pool = connect(filename).await?;
    super::migrations::apply(&pool, filename).await?;

    Ok(pool)
}
