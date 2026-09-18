use serde::{Deserialize, Serialize};
use ureq::{Agent, Error};

pub struct HealthClient {
    pub url: String,
    pub agent: Agent,
}

impl HealthClient {
    pub fn health(&self) -> Result<ProsaHealth, Error> {
        self.agent
            .get(format!("{}/health", self.url))
            .call()?
            .body_mut()
            .read_json::<ProsaHealth>()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaHealth {
    pub status: String,
    pub software: String,
    pub version: String,
}
