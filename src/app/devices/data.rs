use super::models::{DeviceError, LinkedDevice, UnlinkedDevice};
use sqlx::SqlitePool;

pub async fn add_unlinked_device(pool: &SqlitePool, device_id: &str, timestamp: i64) -> () {
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO unlinked_devices (device_id, timestamp)
        VALUES ($1, $2)
        "#,
    )
    .bind(device_id)
    .bind(timestamp)
    .execute(pool)
    .await
    .expect("Failed to add unlinked device");
}

pub async fn remove_unlinked_device(pool: &SqlitePool, device_id: &str) -> Result<(), DeviceError> {
    let result = sqlx::query(
        r#"
        DELETE FROM unlinked_devices
        WHERE device_id = $1
        "#,
    )
    .bind(device_id)
    .execute(pool)
    .await
    .expect("Failed to delete unlinked device");

    if result.rows_affected() == 0 {
        return Err(DeviceError::DeviceNotFound);
    }

    Ok(())
}

pub async fn get_unlinked_device(pool: &SqlitePool, device_id: &str) -> Result<UnlinkedDevice, DeviceError> {
    let device: UnlinkedDevice = sqlx::query_as(
        r#"
        SELECT device_id, timestamp
        FROM unlinked_devices
        WHERE device_id = $1
        "#,
    )
    .bind(device_id)
    .fetch_one(pool)
    .await?;

    Ok(device)
}

pub async fn get_unlinked_devices(pool: &SqlitePool) -> Vec<UnlinkedDevice> {
    let devices: Vec<UnlinkedDevice> = sqlx::query_as(
        r#"
        SELECT device_id, timestamp
        FROM unlinked_devices
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("Failed to get unlinked devices");

    devices
}

pub async fn add_linked_device(pool: &SqlitePool, device_id: &str, api_key: &str) -> Result<(), DeviceError> {
    sqlx::query(
        r#"
        INSERT INTO linked_devices (device_id, api_key)
        VALUES ($1, $2)
        "#,
    )
    .bind(device_id)
    .bind(api_key)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_linked_device(
    pool: &SqlitePool,
    device_id: &str,
    api_key: &str,
) -> Result<(), DeviceError> {
    let result = sqlx::query(
        r#"
        DELETE FROM linked_devices
        WHERE device_id = $1 AND api_key = $2
        "#,
    )
    .bind(device_id)
    .bind(api_key)
    .execute(pool)
    .await
    .expect("Failed to delete linked device");

    if result.rows_affected() == 0 {
        return Err(DeviceError::DeviceNotFound);
    }

    Ok(())
}

pub async fn get_linked_device(pool: &SqlitePool, device_id: &str) -> Option<LinkedDevice> {
    let device: Option<LinkedDevice> = sqlx::query_as(
        r#"
        SELECT device_id, api_key
        FROM linked_devices
        WHERE device_id = $1
        "#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .expect("Failed to get linked device");

    device
}

pub async fn get_linked_devices(pool: &SqlitePool, api_key: &str) -> Vec<String> {
    let devices: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT device_id
        FROM linked_devices
        WHERE api_key = $1
        "#,
    )
    .bind(api_key)
    .fetch_all(pool)
    .await
    .expect("Failed to get linked devices");

    devices
}
