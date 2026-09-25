mod connection;
mod migrations;

pub use connection::{DatabaseError, connect, init};
pub use migrations::{latest_version, revert_to, status};

use sqlx::SqlitePool;
use std::sync::OnceLock;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub fn set_pool(pool: SqlitePool) -> Result<(), DatabaseError> {
    DB_POOL
        .set(pool)
        .map_err(|_| "database pool was already initialized".into())
}

pub fn pool() -> &'static SqlitePool {
    DB_POOL
        .get()
        .expect("database pool accessed before initialization")
}
