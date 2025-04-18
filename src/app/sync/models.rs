use super::service::unix_millis_to_string;
use crate::app::metadata::BookMetadata;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct NewEntitlement {
    pub book_entitlement: BookEntitlement,
    pub reading_state: ReadingState,
    pub book_metadata: BookMetadata,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReadingState {
    pub entitlement_id: String,
    pub created: String,
    pub last_modified: String,
    pub status_info: StatusInfo,
    pub statistics: Statistics,
    pub current_bookmark: CurrentBookmark,
    pub priority_timestamp: String,
}

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatusInfo {
    pub last_modified: String,
    pub status: String,
    pub times_started_reading: u64,
    pub last_time_started_reading: Option<String>,
    pub last_time_finished: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Statistics {
    pub last_modified: String,
    pub spent_reading_minutes: u64,
    pub remaining_time_minutes: u64,
}

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CurrentBookmark {
    pub last_modified: String,
    pub progress_percent: u64,
    pub content_source_progress_percent: u64,
    pub location: Option<Location>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Location {
    pub value: String,
    pub r#type: String,
    pub source: String,
}

//TODO check if I can have tag in location but not source
//TODO check in timestamp being now in each object does not ruin anything
impl ReadingState {
    pub fn new(book_id: &str, status: &str, tag: Option<String>, source: Option<String>) -> Self {
        let now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get time since epoch")
            .as_millis()
            .try_into()
            .expect("Failed to get current timestamp");

        let now = unix_millis_to_string(now);

        let status_info = StatusInfo {
            last_modified: now.clone(),
            status: status.to_string(),
            times_started_reading: 0,
            last_time_started_reading: None,
            last_time_finished: now.clone(),
        };

        let statistics = Statistics {
            last_modified: now.clone(),
            spent_reading_minutes: 0,
            remaining_time_minutes: 0,
        };

        let location = match (tag, source) {
            (Some(tag), Some(source)) => Some(Location {
                value: tag.to_string(),
                r#type: "KoboSpan".to_string(),
                source: source.to_string(),
            }),
            _ => None,
        };

        let current_bookmark = CurrentBookmark {
            last_modified: now.clone(),
            progress_percent: 0,
            content_source_progress_percent: 0,
            location,
        };

        ReadingState {
            entitlement_id: book_id.to_string(),
            created: now.clone(),
            last_modified: now.clone(),
            status_info,
            statistics,
            current_bookmark,
            priority_timestamp: now,
        }
    }
}
