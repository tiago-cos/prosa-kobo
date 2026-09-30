use super::models::{DeviceState, StateError};
use sqlx::SqliteExecutor;

pub async fn get_device_state<'e>(
    db: impl SqliteExecutor<'e>,
    device_id: &str,
    book_id: &str,
) -> Result<Option<DeviceState>, StateError> {
    let state = sqlx::query_as(
        r"
        SELECT status, chapter, span
        FROM device_states
        WHERE device_id = $1 AND book_id = $2
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .fetch_optional(db)
    .await?;

    Ok(state)
}

pub async fn set_device_state<'e>(
    db: impl SqliteExecutor<'e>,
    device_id: &str,
    book_id: &str,
    state: &DeviceState,
) -> Result<(), StateError> {
    sqlx::query(
        r"
        INSERT INTO device_states (device_id, book_id, status, chapter, span)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(device_id, book_id) DO UPDATE
        SET status = excluded.status, chapter = excluded.chapter, span = excluded.span
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .bind(state.status)
    .bind(state.chapter.as_deref())
    .bind(state.span.as_deref())
    .execute(db)
    .await?;

    Ok(())
}

pub async fn remove_device_state<'e>(
    db: impl SqliteExecutor<'e>,
    device_id: &str,
    book_id: &str,
) -> Result<(), StateError> {
    sqlx::query(
        r"
        DELETE FROM device_states
        WHERE device_id = $1 AND book_id = $2
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .execute(db)
    .await?;

    Ok(())
}
