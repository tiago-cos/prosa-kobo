use super::models::{ReadingState, UPDATE_STATE_RESPONSE};
use crate::client::prosa::Client;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde_json::Value;

pub async fn translate_get_state(client: &Client, book_id: &str, api_key: &str) -> ReadingState {
    let state_response = client
        .fetch_state(&book_id, api_key)
        .expect("State response should not fail");

    let status = match state_response.statistics.reading_status.as_ref() {
        "Read" => "Finished".to_string(),
        "Unread" => "ReadyToRead".to_string(),
        status => status.to_string(),
    };

    let state_location = state_response.location.as_ref();

    ReadingState::new(
        &book_id,
        &status,
        state_location.and_then(|l| l.tag.clone()),
        state_location.and_then(|l| l.source.clone()),
    )
}

//TODO verify if having a status: unread book with active location causes bugs
pub async fn translate_update_state(
    client: &Client,
    book_id: &str,
    state: &ReadingState,
    api_key: &str,
) -> Value {
    let location = state.current_bookmark.location.as_ref();
    let status = match state.status_info.status.as_ref() {
        "Finished" => "Read".to_string(),
        "ReadyToRead" => "Unread".to_string(),
        s => s.to_string(),
    };

    let source = location.and_then(|l| {
        let re = Regex::new(r"!!").expect("Failed to create regex");
        let mut matches = re.find_iter(&l.source);

        match matches.next() {
            Some(m) => return Some(l.source[m.end()..].to_string()),
            _ => return Some(l.source.to_string()),
        }
    });

    client
        .update_state(
            &book_id,
            location.and_then(|l| Some(l.value.clone())),
            source,
            &status,
            api_key,
        )
        .expect("State request should not fail");

    let response = &UPDATE_STATE_RESPONSE.replace("{book_id}", book_id);

    serde_json::from_str(&response).expect("Failed to convert to JSON")
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
