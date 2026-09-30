use crate::suite::Problem;
use serde::Deserialize;
use serde_json::{Value, json};

pub struct Middleware {
    url: String,
    http: reqwest::Client,
}

#[derive(Deserialize)]
pub struct Link {
    pub device_id: String,
    pub api_endpoint: String,
}

#[derive(Deserialize)]
struct LinkedDevice {
    device_id: String,
}

impl Middleware {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_owned(),
            http: reqwest::Client::new(),
        }
    }

    /// The endpoint the device is handed names the host it asked for, so the
    /// link is requested as the Kobo will address the middleware.
    pub async fn link(&self, jwt: &str, api_key: &str, kobo_address: &str) -> Result<Link, Problem> {
        let response = self
            .http
            .post(format!("{}/devices/linked", self.url))
            .bearer_auth(jwt)
            .header("Host", kobo_address)
            .json(&json!({ "name": "Device test Kobo", "api_key": api_key }))
            .send()
            .await
            .map_err(|_| Problem::new(format!("prosa-kobo does not answer at {}", self.url)))?;

        Ok(checked(response).await?.json().await?)
    }

    pub async fn is_linked(&self, jwt: &str, user_id: &str, device_id: &str) -> Result<bool, Problem> {
        let response = self
            .http
            .get(format!("{}/devices/linked", self.url))
            .query(&[("user_id", user_id)])
            .bearer_auth(jwt)
            .send()
            .await
            .map_err(|_| Problem::new(format!("prosa-kobo does not answer at {}", self.url)))?;
        let devices: Vec<LinkedDevice> = checked(response).await?.json().await?;

        Ok(devices.iter().any(|device| device.device_id == device_id))
    }

    pub async fn unlink(&self, jwt: &str, device_id: &str) -> Result<(), Problem> {
        let response = self
            .http
            .delete(format!("{}/devices/linked/{device_id}", self.url))
            .bearer_auth(jwt)
            .send()
            .await?;
        checked(response).await?;

        Ok(())
    }

    /// Reads what the device would be sent, through the device's own key.
    /// Only endpoints without side effects are read this way: fetching sync
    /// would spend the device's sync token, and fetching a book's state would
    /// record it as the one the device holds.
    pub async fn as_device(&self, lookup_key: &str, path: &str) -> Result<Value, Problem> {
        Ok(self.get_as_device(lookup_key, path).await?.json().await?)
    }

    pub async fn download_as_device(&self, lookup_key: &str, path: &str) -> Result<Vec<u8>, Problem> {
        Ok(self
            .get_as_device(lookup_key, path)
            .await?
            .bytes()
            .await?
            .to_vec())
    }

    async fn get_as_device(&self, lookup_key: &str, path: &str) -> Result<reqwest::Response, Problem> {
        let response = self
            .http
            .get(format!("{}/{lookup_key}{path}", self.url))
            .send()
            .await?;

        checked(response).await
    }
}

async fn checked(response: reqwest::Response) -> Result<reqwest::Response, Problem> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let path = response.url().path().to_owned();
    let body = response.text().await.unwrap_or_default();
    Err(Problem::new(format!(
        "prosa-kobo answered {status} for {path}: {body}"
    )))
}
