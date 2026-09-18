use serde::{Deserialize, Serialize};
use ureq::{Agent, Error};

pub struct IdentityClient {
    pub url: String,
    pub agent: Agent,
}

impl IdentityClient {
    pub fn identity(&self, api_key: &str) -> Result<ProsaIdentity, Error> {
        self.agent
            .get(format!("{}/auth/me", self.url))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaIdentity>()
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
    pub is_admin: bool,
    pub capabilities: Vec<String>,
    pub key_id: Option<String>,
}
