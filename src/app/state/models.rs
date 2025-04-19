use super::service::unix_millis_to_string;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::time::{SystemTime, UNIX_EPOCH};

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
