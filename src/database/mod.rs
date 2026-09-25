mod connection;
mod migrations;

pub use connection::{DatabaseError, connect, init};
pub use migrations::{latest_version, revert_to, status};

use crate::app::AppState;
use axum::extract::FromRef;
use sqlx::SqlitePool;
use std::sync::Arc;

impl FromRef<AppState> for Arc<SqlitePool> {
    fn from_ref(state: &AppState) -> Arc<SqlitePool> {
        Arc::clone(&state.pool)
    }
}
