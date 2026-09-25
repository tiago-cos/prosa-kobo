use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

pub struct IdentityClient {
    pub url: String,
    pub http: Client,
}

impl IdentityClient {
    pub async fn identity(&self, api_key: &str) -> Result<ProsaIdentity, Error> {
        self.http
            .get(format!("{}/auth/me", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<ProsaIdentity>()
            .await
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProsaAuthType {
    Jwt,
    ApiKey,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaIdentity {
    pub auth_type: ProsaAuthType,
    pub user_id: String,
    pub username: String,
    pub is_admin: bool,
    pub capabilities: Vec<String>,
    pub key_id: Option<String>,
}
