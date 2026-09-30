use crate::{
    app::{error::unmapped, kepub::KoboPosition, kobo_time},
    client::ProsaReadingStatus,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
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
    #[strum(message = "InternalError")]
    #[strum(detailed_message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

impl From<sqlx::Error> for StateError {
    fn from(error: sqlx::Error) -> Self {
        unmapped(&error, StateError::InternalError)
    }
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

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingStatus {
    ReadyToRead,
    Finished,
    #[serde(other)]
    Reading,
}

impl From<ProsaReadingStatus> for ReadingStatus {
    fn from(status: ProsaReadingStatus) -> Self {
        match status {
            ProsaReadingStatus::Unread => ReadingStatus::ReadyToRead,
            ProsaReadingStatus::Reading => ReadingStatus::Reading,
            ProsaReadingStatus::Read => ReadingStatus::Finished,
        }
    }
}

impl From<ReadingStatus> for ProsaReadingStatus {
    fn from(status: ReadingStatus) -> Self {
        match status {
            ReadingStatus::ReadyToRead => ProsaReadingStatus::Unread,
            ReadingStatus::Reading => ProsaReadingStatus::Reading,
            ReadingStatus::Finished => ProsaReadingStatus::Read,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StatusInfo {
    pub last_modified: String,
    pub status: ReadingStatus,
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
    pub fn new(book_id: &str, status: ReadingStatus, position: Option<KoboPosition>) -> Self {
        let now = kobo_time::now();

        let status_info = StatusInfo {
            last_modified: now.clone(),
            status,
            times_started_reading: None,
            last_time_started_reading: None,
            last_time_finished: None,
        };

        let statistics = Statistics {
            last_modified: now.clone(),
            spent_reading_minutes: 0,
            remaining_time_minutes: 0,
        };

        let location = position.map(|position| Location {
            value: position.span,
            r#type: "KoboSpan".to_string(),
            source: position.chapter,
        });

        let current_bookmark = CurrentBookmark {
            last_modified: now.clone(),
            progress_percent: 0,
            content_source_progress_percent: 0,
            location,
        };

        ReadingState {
            entitlement_id: book_id.to_string(),
            created: Some(now.clone()),
            last_modified: now,
            status_info,
            statistics,
            current_bookmark,
            priority_timestamp: None,
        }
    }

    #[must_use]
    pub fn dated(mut self, time: &str) -> Self {
        self.created = Some(time.to_owned());
        for stamp in [
            &mut self.last_modified,
            &mut self.status_info.last_modified,
            &mut self.statistics.last_modified,
            &mut self.current_bookmark.last_modified,
        ] {
            time.clone_into(stamp);
        }

        self
    }

    pub fn for_removed_book() -> Self {
        ReadingState::new("placeholder", ReadingStatus::Reading, None)
    }
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct DeviceState {
    pub status: ReadingStatus,
    pub chapter: Option<String>,
    pub span: Option<String>,
}

impl DeviceState {
    pub fn of(state: &ReadingState) -> Self {
        let status = state.status_info.status;
        let position = match status {
            ReadingStatus::Reading => state.current_bookmark.location.as_ref().map(Location::position),
            ReadingStatus::ReadyToRead | ReadingStatus::Finished => None,
        };

        Self {
            status,
            chapter: position.as_ref().map(|position| position.chapter.clone()),
            span: position.map(|position| position.span),
        }
    }

    pub fn needs_sync(&self, stored: Option<&DeviceState>) -> bool {
        match (self.status, stored) {
            (ReadingStatus::Reading, Some(stored)) => stored.status != ReadingStatus::Reading,
            (ReadingStatus::Reading, None) => true,
            (_, stored) => stored != Some(self),
        }
    }
}

impl Location {
    // The device names the chapter relative to the book file it downloaded, as
    // `<book>.kepub.epub!!OEBPS/chapter.xhtml`; only the tail names a document.
    pub fn position(&self) -> KoboPosition {
        let chapter = match self.source.split_once("!!") {
            Some((_, source)) => source,
            None => self.source.as_str(),
        };

        KoboPosition::new(chapter, &self.value, 0)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateStateRequest {
    pub reading_states: Vec<ReadingState>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateStateResponse {
    pub request_result: &'static str,
    pub update_results: Vec<UpdateResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateResult {
    pub entitlement_id: String,
    pub status_info_result: UpdateOutcome,
    pub statistics_result: UpdateOutcome,
    pub current_bookmark_result: UpdateOutcome,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateOutcome {
    pub result: &'static str,
}

impl UpdateStateResponse {
    pub fn success(book_id: &str) -> Self {
        Self {
            request_result: "Success",
            update_results: vec![UpdateResult {
                entitlement_id: book_id.to_owned(),
                status_info_result: UpdateOutcome { result: "Success" },
                statistics_result: UpdateOutcome { result: "Success" },
                current_bookmark_result: UpdateOutcome { result: "Success" },
            }],
        }
    }
}

#[derive(Deserialize)]
pub struct RatingQuery {
    #[serde(rename = "ProductIds")]
    pub product_ids: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RatingResponse {
    pub items: Vec<Rating>,
    pub total_page_count: u8,
    pub current_page_index: u8,
}

impl RatingResponse {
    pub fn new(book_id: &str, rating: Option<f32>) -> Self {
        let items = match rating {
            None => vec![],
            Some(rating) => vec![Rating::new(book_id, kobo_rating(rating))],
        };

        RatingResponse {
            items,
            total_page_count: 1,
            current_page_index: 1,
        }
    }
}

pub fn prosa_rating(rating: u8) -> Option<f32> {
    match rating {
        0 => None,
        rating => Some(rating.into()),
    }
}

fn kobo_rating(rating: f32) -> u8 {
    rating.round().clamp(0.0, f32::from(u8::MAX)) as u8
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

#[derive(Serialize)]
pub struct EmptyObject {}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReviewsResponse {
    pub review_summary: EmptyObject,
    pub cursor: &'static str,
    pub items: &'static [EmptyObject],
    pub total_page_count: u8,
    pub current_page_index: u8,
}

pub const REVIEWS_RESPONSE: ReviewsResponse = ReviewsResponse {
    review_summary: EmptyObject {},
    cursor: "1",
    items: &[],
    total_page_count: 10,
    current_page_index: 1,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn names_each_status_as_the_device_does() {
        for (status, name) in [
            (ReadingStatus::ReadyToRead, "ReadyToRead"),
            (ReadingStatus::Reading, "Reading"),
            (ReadingStatus::Finished, "Finished"),
        ] {
            assert_eq!(serde_json::to_value(status).ok(), Some(json!(name)));
            assert_eq!(
                serde_json::from_value::<ReadingStatus>(json!(name)).ok(),
                Some(status)
            );
        }
    }

    #[test]
    fn reads_a_status_it_does_not_know_as_reading() {
        assert_eq!(
            serde_json::from_value::<ReadingStatus>(json!("Paused")).ok(),
            Some(ReadingStatus::Reading)
        );
    }

    #[test]
    fn maps_to_prosa_and_back_unchanged() {
        for status in [
            ProsaReadingStatus::Unread,
            ProsaReadingStatus::Reading,
            ProsaReadingStatus::Read,
        ] {
            assert_eq!(ProsaReadingStatus::from(ReadingStatus::from(status)), status);
        }
    }

    fn device_state(status: ReadingStatus, span: Option<&str>) -> DeviceState {
        let position = span.map(|span| KoboPosition::new("OEBPS/chapter.xhtml", span, 0));

        DeviceState::of(&ReadingState::new("book", status, position))
    }

    #[test]
    fn holds_a_position_only_while_a_book_is_being_read() {
        assert_eq!(
            device_state(ReadingStatus::Reading, Some("kobo.1.1"))
                .span
                .as_deref(),
            Some("kobo.1.1")
        );
        assert_eq!(device_state(ReadingStatus::Finished, Some("kobo.1.1")).span, None);
        assert_eq!(
            device_state(ReadingStatus::ReadyToRead, Some("kobo.1.1")).span,
            None
        );
    }

    #[test]
    fn reads_the_chapter_a_device_names_through_its_book_file() {
        let reported = Location {
            value: "kobo.1.1".to_owned(),
            r#type: "KoboSpan".to_owned(),
            source: "book.kepub.epub!!OEBPS/chapter.xhtml".to_owned(),
        };
        let mut state = ReadingState::new("book", ReadingStatus::Reading, None);
        state.current_bookmark.location = Some(reported);

        assert_eq!(
            DeviceState::of(&state),
            device_state(ReadingStatus::Reading, Some("kobo.1.1"))
        );
    }

    #[test]
    fn leaves_a_new_position_to_a_device_already_reading() {
        let moved = device_state(ReadingStatus::Reading, Some("kobo.9.1"));

        assert!(!moved.needs_sync(Some(&device_state(ReadingStatus::Reading, Some("kobo.1.1")))));
        assert!(moved.needs_sync(Some(&device_state(ReadingStatus::ReadyToRead, None))));
        assert!(moved.needs_sync(Some(&device_state(ReadingStatus::Finished, None))));
        assert!(moved.needs_sync(None));
    }

    #[test]
    fn sends_a_book_put_aside_only_to_a_device_that_does_not_hold_it_so() {
        for status in [ReadingStatus::ReadyToRead, ReadingStatus::Finished] {
            let put_aside = device_state(status, None);

            assert!(!put_aside.needs_sync(Some(&device_state(status, None))));
            assert!(put_aside.needs_sync(Some(&device_state(ReadingStatus::Reading, Some("kobo.1.1")))));
            assert!(put_aside.needs_sync(None));
        }
        assert!(
            device_state(ReadingStatus::Finished, None)
                .needs_sync(Some(&device_state(ReadingStatus::ReadyToRead, None)))
        );
    }
}
