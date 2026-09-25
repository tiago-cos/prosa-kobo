use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

pub struct HealthClient {
    pub url: String,
    pub http: Client,
}

impl HealthClient {
    pub async fn health(&self) -> Result<ProsaHealth, Error> {
        self.http
            .get(format!("{}/health", self.url))
            .send()
            .await?
            .error_for_status()?
            .json::<ProsaHealth>()
            .await
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaHealth {
    pub status: String,
    pub software: String,
    pub version: String,
}
