use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};

pub type DatabaseError = Box<dyn std::error::Error + Send + Sync>;

pub async fn connect(filename: &str) -> Result<Pool<Sqlite>, DatabaseError> {
    let options = SqliteConnectOptions::new()
        .filename(filename)
        .create_if_missing(true);

    Ok(SqlitePool::connect_with(options).await?)
}

pub async fn init(filename: &str) -> Result<Pool<Sqlite>, DatabaseError> {
    let pool = connect(filename).await?;
    super::migrations::apply(&pool, filename).await?;

    Ok(pool)
}
