use super::{
    data,
    models::{DeviceError, LinkedDevice},
};
use crate::{
    app::error::KoboError,
    client::prosa::{ClientError, ProsaApi},
    database::pool,
};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::RngCore;

const LOOKUP_KEY_SIZE: usize = 32;

pub async fn link_device(
    client: &dyn ProsaApi,
    user_id: &str,
    name: &str,
    api_key: &str,
) -> Result<(String, String), KoboError> {
    if name.trim().is_empty() {
        return Err(DeviceError::InvalidDeviceName.into());
    }

    verify_api_key(client, api_key)?;

    let device_id = generate_secret(16);
    let lookup_key = generate_secret(LOOKUP_KEY_SIZE);

    data::add_linked_device(pool(), &device_id, &lookup_key, user_id, name, api_key).await?;

    Ok((device_id, lookup_key))
}

pub async fn unlink_device(device_id: &str) -> Result<(), KoboError> {
    data::remove_linked_device(pool(), device_id).await?;

    Ok(())
}

pub async fn get_linked_device(device_id: &str) -> Option<LinkedDevice> {
    data::get_linked_device(pool(), device_id).await
}

pub async fn get_device_by_lookup_key(lookup_key: &str) -> Option<LinkedDevice> {
    data::get_device_by_lookup_key(pool(), lookup_key).await
}

pub async fn get_device_by_client_id(client_device_id: &str) -> Option<LinkedDevice> {
    data::get_device_by_client_id(pool(), client_device_id).await
}

pub async fn claim_client_device_id(device_id: &str, client_device_id: &str) {
    data::claim_client_device_id(pool(), device_id, client_device_id).await;
}

pub async fn get_linked_devices(user_id: Option<&str>) -> Vec<LinkedDevice> {
    data::get_linked_devices(pool(), user_id).await
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

fn generate_secret(size: usize) -> String {
    let mut bytes = vec![0u8; size];
    rand::rng().fill_bytes(&mut bytes);

    BASE64_URL_SAFE_NO_PAD.encode(bytes)
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
            username: "a-name".to_owned(),
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
