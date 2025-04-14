use super::{
    data,
    models::{DeviceError, LinkedDevice, UnlinkedDevice},
};
use crate::app::error::KoboError;
use base64::{prelude::BASE64_STANDARD, Engine};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

pub async fn add_unlinked_device(pool: &SqlitePool, device_id: &str, timestamp: i64) -> () {
    let linked_device = data::get_linked_device(pool, device_id).await;
    if linked_device.is_none() {
        data::add_unlinked_device(pool, device_id, timestamp).await;
    }
}

pub async fn get_unlinked_devices(pool: &SqlitePool) -> Vec<UnlinkedDevice> {
    data::get_unlinked_devices(pool).await
}

pub async fn link_device(pool: &SqlitePool, device_id: &str, api_key: &str) -> Result<(), KoboError> {
    if !is_valid_api_key(api_key) {
        return Err(DeviceError::InvalidApiKey.into());
    }

    data::get_unlinked_device(pool, device_id).await?;
    data::add_linked_device(pool, device_id, api_key).await?;
    data::remove_unlinked_device(pool, device_id).await?;

    Ok(())
}

pub async fn unlink_device(
    pool: &SqlitePool,
    device_id: &str,
    api_key: &str,
    timestamp: i64,
) -> Result<(), KoboError> {
    if !is_valid_api_key(api_key) {
        return Err(DeviceError::InvalidApiKey.into());
    }

    data::remove_linked_device(pool, device_id, api_key).await?;
    data::add_unlinked_device(pool, device_id, timestamp).await;

    Ok(())
}

pub async fn get_linked_devices(pool: &SqlitePool, api_key: &str) -> Result<Vec<String>, KoboError> {
    if !is_valid_api_key(api_key) {
        return Err(DeviceError::InvalidApiKey.into());
    }

    Ok(data::get_linked_devices(pool, api_key).await)
}

pub async fn get_linked_device(pool: &SqlitePool, device_id: &str) -> Option<LinkedDevice> {
    data::get_linked_device(pool, device_id).await
}

pub async fn generate_device_id(device_id: &str, user_key: &str) -> String {
    let digest = Sha256::digest(device_id.to_owned() + user_key);
    BASE64_STANDARD.encode(digest)
}

fn is_valid_api_key(key: &str) -> bool {
    if key.trim().is_empty() {
        return false;
    }

    return key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');
}
