use super::models::AnnotationError;
use sqlx::SqliteExecutor;

pub async fn get_etag<'e>(
    db: impl SqliteExecutor<'e>,
    book_id: &str,
) -> Result<Option<String>, AnnotationError> {
    let etag = sqlx::query_scalar(
        r"
        SELECT etag
        FROM etags
        WHERE book_id = $1
        ",
    )
    .bind(book_id)
    .fetch_optional(db)
    .await?;

    Ok(etag)
}

pub async fn update_etag<'e>(
    db: impl SqliteExecutor<'e>,
    book_id: &str,
    etag: &str,
) -> Result<(), AnnotationError> {
    sqlx::query(
        r"
        INSERT OR REPLACE INTO etags (book_id, etag)
        VALUES ($1, $2)
        ",
    )
    .bind(book_id)
    .bind(etag)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_etag<'e>(db: impl SqliteExecutor<'e>, book_id: &str) -> Result<(), AnnotationError> {
    sqlx::query(
        r"
        DELETE FROM etags
        WHERE book_id = $1
        ",
    )
    .bind(book_id)
    .execute(db)
    .await?;

    Ok(())
}
