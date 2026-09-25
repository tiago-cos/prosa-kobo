use super::models::CoverError;
use sqlx::SqliteExecutor;

pub async fn get_version<'e>(
    db: impl SqliteExecutor<'e>,
    device_id: &str,
    book_id: &str,
) -> Result<Option<i64>, CoverError> {
    let version = sqlx::query_scalar(
        r"
        SELECT version
        FROM cover_versions
        WHERE device_id = $1 AND book_id = $2
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .fetch_optional(db)
    .await?;

    Ok(version)
}

pub async fn bump_version<'e>(
    db: impl SqliteExecutor<'e>,
    device_id: &str,
    book_id: &str,
) -> Result<(), CoverError> {
    sqlx::query(
        r"
        INSERT INTO cover_versions (device_id, book_id, version)
        VALUES ($1, $2, 1)
        ON CONFLICT(device_id, book_id) DO UPDATE SET version = version + 1
        ",
    )
    .bind(device_id)
    .bind(book_id)
    .execute(db)
    .await?;

    Ok(())
}
