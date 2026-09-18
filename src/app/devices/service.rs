use super::{
    data,
    models::{DeviceError, LinkedDevice, UnlinkedDevice},
};
use crate::{
    CONFIG,
    app::error::KoboError,
    client::prosa::{ClientError, ProsaApi},
};
use base64::{Engine, prelude::BASE64_URL_SAFE};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn add_unlinked_device(pool: &SqlitePool, device_id: &str) -> () {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;

    data::add_unlinked_device(pool, device_id, now).await;
}

pub async fn get_unlinked_devices(pool: &SqlitePool) -> Vec<UnlinkedDevice> {
    remove_expired_unlinked_devices(pool).await;

    data::get_unlinked_devices(pool).await
}

pub async fn link_device(
    pool: &SqlitePool,
    client: &dyn ProsaApi,
    device_id: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    verify_api_key(client, api_key)?;

    remove_expired_unlinked_devices(pool).await;

    if data::get_linked_device(pool, device_id).await.is_some() {
        return Err(DeviceError::DeviceAlreadyLinked.into());
    }

    data::remove_unlinked_device(pool, device_id).await?;
    data::add_linked_device(pool, device_id, api_key).await?;

    Ok(())
}

pub async fn unlink_device(pool: &SqlitePool, device_id: &str, api_key: &str) -> Result<(), KoboError> {
    if !is_valid_api_key(api_key) {
        return Err(DeviceError::InvalidApiKey.into());
    }

    if data::get_unlinked_device(pool, device_id).await.is_some() {
        return Err(DeviceError::DeviceAlreadyUnlinked.into());
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;

    data::remove_linked_device(pool, device_id, api_key).await?;
    data::add_unlinked_device(pool, device_id, now).await;

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

pub async fn get_unlinked_device(pool: &SqlitePool, device_id: &str) -> Option<UnlinkedDevice> {
    remove_expired_unlinked_devices(pool).await;

    data::get_unlinked_device(pool, device_id).await
}

async fn remove_expired_unlinked_devices(pool: &SqlitePool) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;

    data::remove_expired_unlinked_devices(pool, now - CONFIG.devices.unlinked_expiration).await;
}

pub fn generate_device_id(device_id: &str, user_key: &str) -> String {
    let digest = Sha256::digest(device_id.to_owned() + user_key);
    BASE64_URL_SAFE.encode(digest)
}

fn verify_api_key(client: &dyn ProsaApi, api_key: &str) -> Result<(), KoboError> {
    if !is_valid_api_key(api_key) {
        return Err(DeviceError::InvalidApiKey.into());
    }

    match client.identity(api_key) {
        Ok(_) => Ok(()),
        Err(ClientError::Unauthorized) => Err(DeviceError::InvalidApiKey.into()),
        Err(ClientError::Forbidden) => Err(DeviceError::InsufficientApiKey.into()),
        Err(error) => Err(error.into()),
    }
}

fn is_valid_api_key(key: &str) -> bool {
    if key.trim().is_empty() {
        return false;
    }

    key.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{
        identity::{ProsaAuthType, ProsaIdentity},
        mock::{MockProsaClient, ProsaMethod},
    };

    const KEY: &str = "dXNhYmxlLWtleQ==";

    fn identity() -> ProsaIdentity {
        ProsaIdentity {
            auth_type: ProsaAuthType::ApiKey,
            user_id: "a-user".to_owned(),
            is_admin: false,
            capabilities: vec!["Read".to_owned(), "Create".to_owned()],
            key_id: Some("a-key".to_owned()),
        }
    }

    fn refusal(result: Result<(), KoboError>) -> String {
        result
            .expect_err("Expected the key to be refused")
            .get_message()
            .expect("Expected an error code")
            .to_owned()
    }

    #[test]
    fn accepts_a_key_prosa_recognises() {
        let client = MockProsaClient::new();
        client.seed_identity(KEY, identity());

        verify_api_key(&client, KEY).expect("Expected the key to be accepted");

        assert_eq!(client.call_count(ProsaMethod::Identity), 1);
    }

    #[test]
    fn refuses_a_key_prosa_does_not_know() {
        let client = MockProsaClient::new();

        assert_eq!(refusal(verify_api_key(&client, KEY)), "InvalidApiKey");
    }

    #[test]
    fn refuses_a_key_without_read_access() {
        let client = MockProsaClient::new();
        client.fail(ProsaMethod::Identity, ClientError::Forbidden);

        assert_eq!(refusal(verify_api_key(&client, KEY)), "InsufficientApiKey");
    }

    #[test]
    fn does_not_ask_prosa_about_a_malformed_key() {
        let client = MockProsaClient::new();

        assert_eq!(refusal(verify_api_key(&client, "not a key!")), "InvalidApiKey");
        assert_eq!(client.call_count(ProsaMethod::Identity), 0);
    }
}
