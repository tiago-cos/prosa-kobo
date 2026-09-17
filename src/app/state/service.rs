use super::models::{ReadingState, UPDATE_STATE_RESPONSE};
use crate::{
    app::{error::KoboError, state::models::RatingResponse},
    client::{ProsaLocation, ProsaReadingStatus, prosa::ProsaApi},
};
use chrono::{DateTime, Utc};
use serde_json::Value;

pub fn translate_get_state(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<ReadingState, KoboError> {
    let state_response = client.fetch_state(book_id, api_key)?;

    let status = match state_response.statistics.reading_status {
        ProsaReadingStatus::Read => "Finished",
        ProsaReadingStatus::Unread => "ReadyToRead",
        ProsaReadingStatus::Reading => "Reading",
    };

    // A bookmark we cannot parse is no bookmark: the device gets the book
    // without a reading position rather than a broken one.
    let location = state_response
        .location
        .and_then(|location| location.parse::<ProsaLocation>().ok());

    let state = ReadingState::new(
        book_id,
        status,
        location.as_ref().map(ProsaLocation::fragment),
        location.map(|location| location.source),
    );

    Ok(state)
}

pub fn translate_update_state(
    client: &dyn ProsaApi,
    book_id: &str,
    state: &ReadingState,
    api_key: &str,
) -> Result<Value, KoboError> {
    let status = match state.status_info.status.as_str() {
        "Finished" => ProsaReadingStatus::Read,
        "ReadyToRead" => ProsaReadingStatus::Unread,
        _ => ProsaReadingStatus::Reading,
    };

    // The device names the chapter relative to the book file it downloaded,
    // as `<book>.kepub.epub!!OEBPS/chapter.xhtml`; Prosa wants the tail.
    let location = state.current_bookmark.location.as_ref().map(|location| {
        let source = match location.source.split_once("!!") {
            Some((_, source)) => source,
            None => location.source.as_str(),
        };

        format!("{source}#{}", location.value)
    });

    client.patch_state(book_id, location.as_deref(), status, api_key)?;

    let response = &UPDATE_STATE_RESPONSE.replace("{book_id}", book_id);
    let response = serde_json::from_str(response).expect("Failed to convert to JSON");

    Ok(response)
}

pub fn translate_update_rating(
    client: &dyn ProsaApi,
    book_id: &str,
    rating: u8,
    api_key: &str,
) -> Result<(), KoboError> {
    client.update_rating(book_id, rating, api_key)?;

    Ok(())
}

pub fn translate_get_rating(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<RatingResponse, KoboError> {
    let rating = client.fetch_rating(book_id, api_key)?;

    Ok(RatingResponse::new(book_id, rating))
}

pub fn unix_millis_to_string(timestamp_millis: i64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp_millis(timestamp_millis)
        .expect("Failed to convert timesstamp to string");

    let formatted = format!(
        "{}.{:07}Z",
        datetime.format("%Y-%m-%dT%H:%M:%S"),
        datetime.timestamp_subsec_nanos() / 100
    );

    formatted
}
