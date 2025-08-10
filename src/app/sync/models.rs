use super::service::unix_millis_to_string;
use crate::app::{
    metadata::BookMetadata,
    shelves::models::{DeletedShelfResponse, NewShelfResponse},
    state::models::ReadingState,
};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum SyncItem {
    Entitlement(NewEntitlementResponse),
    NewShelf(NewShelfResponse),
    DeletedShelf(DeletedShelfResponse),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct NewEntitlementResponse {
    pub new_entitlement: NewEntitlement,
}

impl NewEntitlementResponse {
    pub fn new(
        book_entitlement: BookEntitlement,
        reading_state: ReadingState,
        book_metadata: BookMetadata,
    ) -> Self {
        let new_entitlement = NewEntitlement {
            book_entitlement,
            reading_state,
            book_metadata,
        };
        NewEntitlementResponse { new_entitlement }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct NewEntitlement {
    pub book_entitlement: BookEntitlement,
    pub reading_state: ReadingState,
    pub book_metadata: BookMetadata,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BookEntitlement {
    pub active_period: ActivePeriod,
    pub is_removed: bool,
    pub status: String,
    pub accessibility: String,
    pub cross_revision_id: String,
    pub revision_id: String,
    pub is_hidden_from_archive: bool,
    pub id: String,
    pub created: String,
    pub last_modified: String,
    pub is_locked: bool,
    pub origin_category: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ActivePeriod {
    pub from: String,
}

impl BookEntitlement {
    pub fn new(book_id: &str, is_removed: bool) -> Self {
        let now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get time since epoch")
            .as_millis()
            .try_into()
            .expect("Failed to get current timestamp");

        let now = unix_millis_to_string(now);

        BookEntitlement {
            active_period: ActivePeriod { from: now.clone() },
            is_removed,
            status: "Active".to_string(),
            accessibility: "Full".to_string(),
            cross_revision_id: book_id.to_string(),
            revision_id: book_id.to_string(),
            is_hidden_from_archive: is_removed,
            id: book_id.to_string(),
            created: now.clone(),
            last_modified: now,
            is_locked: false,
            origin_category: "Purchased".to_string(),
        }
    }
}
