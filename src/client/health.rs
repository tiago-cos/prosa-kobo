use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaHealth {
    pub status: String,
    pub software: String,
    pub version: String,
}
