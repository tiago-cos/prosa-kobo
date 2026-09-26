use serde::{Deserialize, Serialize};

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
