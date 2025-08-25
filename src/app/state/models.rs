use super::service::unix_millis_to_string;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::time::{SystemTime, UNIX_EPOCH};
use strum_macros::{EnumMessage, EnumProperty};

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum StateError {
    #[strum(message = "MissingBookId")]
    #[strum(detailed_message = "A book ID must be provided.")]
    #[strum(props(StatusCode = "400"))]
    MissingProductId,
    #[strum(message = "MissingState")]
    #[strum(detailed_message = "A state must be provided.")]
    #[strum(props(StatusCode = "400"))]
    MissingState,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ReadingState {
    pub entitlement_id: String,
    pub created: Option<String>,
    pub last_modified: String,
    pub status_info: StatusInfo,
    pub statistics: Statistics,
    pub current_bookmark: CurrentBookmark,
    pub priority_timestamp: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StatusInfo {
    pub last_modified: String,
    pub status: String,
    pub times_started_reading: Option<u64>,
    pub last_time_started_reading: Option<String>,
    pub last_time_finished: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Statistics {
    pub last_modified: String,
    pub spent_reading_minutes: u64,
    pub remaining_time_minutes: u64,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CurrentBookmark {
    pub last_modified: String,
    pub progress_percent: u64,
    pub content_source_progress_percent: u64,
    pub location: Option<Location>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Location {
    pub value: String,
    pub r#type: String,
    pub source: String,
}

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
            times_started_reading: None,
            last_time_started_reading: None,
            last_time_finished: None,
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
            created: Some(now.clone()),
            last_modified: now.clone(),
            status_info,
            statistics,
            current_bookmark,
            priority_timestamp: None,
        }
    }
}

impl Default for ReadingState {
    fn default() -> Self {
        ReadingState::new("placeholder", "Reading", None, None)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateStateRequest {
    pub reading_states: Vec<ReadingState>,
}

pub const UPDATE_STATE_RESPONSE: &str = r#"
{
  "RequestResult": "Success",
  "UpdateResults": [
    {
      "EntitlementId": "{book_id}",
      "StatusInfoResult": {
        "Result": "Success"
      },
      "StatisticsResult": {
        "Result": "Success"
      },
      "CurrentBookmarkResult": {
        "Result": "Success"
      }
    }
  ]
}
"#;

#[skip_serializing_none]
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RatingResponse {
    pub items: Vec<Rating>,
    pub total_page_count: u8,
    pub current_page_index: u8,
}

impl RatingResponse {
    pub fn new(book_id: &str, rating: Option<u8>) -> Self {
        let items = match rating {
            None => vec![],
            Some(r) => vec![Rating::new(book_id, r)],
        };

        RatingResponse {
            items,
            total_page_count: 1,
            current_page_index: 1,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Rating {
    pub id: String,
    pub revision_id: String,
    pub product_id: String,
    pub cross_revision_id: String,
    pub publication_id: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub author_display_name: Option<String>,
    pub rating: u8,
    pub creation_date: Option<String>,
    pub likes: u8,
    pub dislikes: u8,
}

impl Rating {
    pub fn new(book_id: &str, rating: u8) -> Self {
        Rating {
            id: book_id.to_string(),
            revision_id: book_id.to_string(),
            product_id: book_id.to_string(),
            cross_revision_id: book_id.to_string(),
            publication_id: "00000000-0000-0000-0000-000000000000".to_string(),
            title: None,
            body: Some(String::new()),
            author_display_name: None,
            rating,
            creation_date: None,
            likes: 0,
            dislikes: 0,
        }
    }
}

pub const REVIEWS_MOCK_RESPONSE: &str = r#"
{
    "ReviewSummary": {},
    "Cursor": "1",
    "Items": [],
    "TotalPageCount": 10,
    "CurrentPageIndex": 1
}
"#;
