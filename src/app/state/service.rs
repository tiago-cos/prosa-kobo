use super::models::{ReadingState, UPDATE_STATE_RESPONSE};
use crate::{
    app::{error::KoboError, state::models::RatingResponse},
    client::prosa::Client,
};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde_json::Value;

pub fn translate_get_state(client: &Client, book_id: &str, api_key: &str) -> Result<ReadingState, KoboError> {
    let state_response = client.fetch_state(book_id, api_key)?;

    let status = match state_response.statistics.reading_status.as_ref() {
        "Read" => "Finished".to_string(),
        "Unread" => "ReadyToRead".to_string(),
        status => status.to_string(),
    };

    let state_location = state_response.location.as_ref();

    let state = ReadingState::new(
        book_id,
        &status,
        state_location.and_then(|l| l.tag.clone()),
        state_location.and_then(|l| l.source.clone()),
    );

    Ok(state)
}

pub fn translate_update_state(
    client: &Client,
    book_id: &str,
    state: &ReadingState,
    api_key: &str,
) -> Result<Value, KoboError> {
    let location = state.current_bookmark.location.as_ref();
    let status = match state.status_info.status.as_ref() {
        "Finished" => "Read".to_string(),
        "ReadyToRead" => "Unread".to_string(),
        s => s.to_string(),
    };

    let source = location.map(|l| {
        let re = Regex::new(r"!!").expect("Failed to create regex");
        let mut matches = re.find_iter(&l.source);

        match matches.next() {
            Some(m) => l.source[m.end()..].to_string(),
            _ => l.source.to_string(),
        }
    });

    client.patch_state(
        book_id,
        location.map(|l| l.value.clone()),
        source,
        &status,
        api_key,
    )?;

    let response = &UPDATE_STATE_RESPONSE.replace("{book_id}", book_id);
    let response = serde_json::from_str(response).expect("Failed to convert to JSON");

    Ok(response)
}

pub fn translate_update_rating(
    client: &Client,
    book_id: &str,
    rating: u8,
    api_key: &str,
) -> Result<(), KoboError> {
    client.update_rating(book_id, rating, api_key)?;

    Ok(())
}

pub fn translate_get_rating(
    client: &Client,
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
