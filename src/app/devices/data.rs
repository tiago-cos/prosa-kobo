use super::models::{DeviceError, LinkedDevice};
use sqlx::{Acquire, Sqlite, SqliteExecutor};

pub async fn add_linked_device<'e>(
    db: impl SqliteExecutor<'e>,
    device_id: &str,
    lookup_key: &str,
    user_id: &str,
    name: &str,
    api_key: &str,
) -> Result<(), DeviceError> {
    sqlx::query(
        r"
        INSERT INTO linked_devices (device_id, lookup_key, user_id, name, api_key)
        VALUES ($1, $2, $3, $4, $5)
        ",
    )
    .bind(device_id)
    .bind(lookup_key)
    .bind(user_id)
    .bind(name)
    .bind(api_key)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn remove_linked_device<'e>(
    db: impl SqliteExecutor<'e>,
    device_id: &str,
) -> Result<(), DeviceError> {
    let result = sqlx::query(
        r"
        DELETE FROM linked_devices
        WHERE device_id = $1
        ",
    )
    .bind(device_id)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DeviceError::DeviceNotFound);
    }

    Ok(())
}

pub async fn get_linked_device<'e>(
    db: impl SqliteExecutor<'e>,
    device_id: &str,
) -> Result<Option<LinkedDevice>, DeviceError> {
    let device: Option<LinkedDevice> = sqlx::query_as(
        r"
        SELECT device_id, lookup_key, user_id, name, api_key, client_device_id
        FROM linked_devices
        WHERE device_id = $1
        ",
    )
    .bind(device_id)
    .fetch_optional(db)
    .await?;

    Ok(device)
}

pub async fn get_device_by_lookup_key<'e>(
    db: impl SqliteExecutor<'e>,
    lookup_key: &str,
) -> Result<Option<LinkedDevice>, DeviceError> {
    let device: Option<LinkedDevice> = sqlx::query_as(
        r"
        SELECT device_id, lookup_key, user_id, name, api_key, client_device_id
        FROM linked_devices
        WHERE lookup_key = $1
        ",
    )
    .bind(lookup_key)
    .fetch_optional(db)
    .await?;

    Ok(device)
}

pub async fn get_linked_devices<'e>(
    db: impl SqliteExecutor<'e>,
    user_id: Option<&str>,
) -> Result<Vec<LinkedDevice>, DeviceError> {
    let devices: Vec<LinkedDevice> = sqlx::query_as(
        r"
        SELECT device_id, lookup_key, user_id, name, api_key, client_device_id
        FROM linked_devices
        WHERE $1 IS NULL OR user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(devices)
}

pub async fn get_device_by_client_id<'e>(
    db: impl SqliteExecutor<'e>,
    client_device_id: &str,
) -> Result<Option<LinkedDevice>, DeviceError> {
    let device = sqlx::query_as(
        r"
        SELECT device_id, lookup_key, user_id, name, api_key, client_device_id
        FROM linked_devices
        WHERE client_device_id = $1
        ",
    )
    .bind(client_device_id)
    .fetch_optional(db)
    .await?;

    Ok(device)
}

pub async fn claim_client_device_id<'a>(
    db: impl Acquire<'a, Database = Sqlite>,
    device_id: &str,
    client_device_id: &str,
) -> Result<(), DeviceError> {
    let mut transaction = db.begin().await?;

    sqlx::query(
        r"
        UPDATE linked_devices
        SET client_device_id = NULL
        WHERE client_device_id = $1 AND device_id != $2
        ",
    )
    .bind(client_device_id)
    .bind(device_id)
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        r"
        UPDATE linked_devices
        SET client_device_id = $1
        WHERE device_id = $2
        ",
    )
    .bind(client_device_id)
    .bind(device_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}
