use super::models::{DeviceError, LinkedDevice, UnlinkedDevice};
use sqlx::SqlitePool;

pub async fn add_unlinked_device(pool: &SqlitePool, device_id: &str, timestamp: i64) -> () {
    sqlx::query(
        r"
        INSERT INTO unlinked_devices (device_id, timestamp)
        VALUES ($1, $2)
        ",
    )
    .bind(device_id)
    .bind(timestamp)
    .execute(pool)
    .await
    .expect("Failed to add unlinked device");
}

pub async fn remove_unlinked_device(pool: &SqlitePool, device_id: &str) -> Result<(), DeviceError> {
    let result = sqlx::query(
        r"
        DELETE FROM unlinked_devices
        WHERE device_id = $1
        ",
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

pub async fn remove_expired_unlinked_devices(pool: &SqlitePool, cutoff: i64) {
    sqlx::query(
        r"
        DELETE FROM unlinked_devices
        WHERE timestamp < $1
        ",
    )
    .bind(cutoff)
    .execute(pool)
    .await
    .expect("Failed to remove expired unlinked devices");
}

pub async fn get_unlinked_device(pool: &SqlitePool, device_id: &str) -> Option<UnlinkedDevice> {
    let device: Option<UnlinkedDevice> = sqlx::query_as(
        r"
        SELECT device_id, timestamp
        FROM unlinked_devices
        WHERE device_id = $1
        ",
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .expect("Failed to fetch unlinked device");

    device
}

pub async fn get_unlinked_devices(pool: &SqlitePool) -> Vec<UnlinkedDevice> {
    let devices: Vec<UnlinkedDevice> = sqlx::query_as(
        r"
        SELECT device_id, timestamp
        FROM unlinked_devices
        ",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to get unlinked devices");

    devices
}

pub async fn add_linked_device(
    pool: &SqlitePool,
    device_id: &str,
    user_id: &str,
    api_key: &str,
) -> Result<(), DeviceError> {
    sqlx::query(
        r"
        INSERT INTO linked_devices (device_id, user_id, api_key)
        VALUES ($1, $2, $3)
        ",
    )
    .bind(device_id)
    .bind(user_id)
    .bind(api_key)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_linked_device(pool: &SqlitePool, device_id: &str) -> Result<(), DeviceError> {
    let result = sqlx::query(
        r"
        DELETE FROM linked_devices
        WHERE device_id = $1
        ",
    )
    .bind(device_id)
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
        r"
        SELECT device_id, user_id, api_key
        FROM linked_devices
        WHERE device_id = $1
        ",
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .expect("Failed to get linked device");

    device
}

pub async fn get_linked_devices(pool: &SqlitePool, user_id: Option<&str>) -> Vec<LinkedDevice> {
    let devices: Vec<LinkedDevice> = sqlx::query_as(
        r"
        SELECT device_id, user_id, api_key
        FROM linked_devices
        WHERE $1 IS NULL OR user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .expect("Failed to get linked devices");

    devices
}
