use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaBookFileMetadata {
    pub owner_id: String,
    pub file_size: u64,
}
