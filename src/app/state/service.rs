use super::models::{EventResponse, ReadingState, UPDATE_STATE_RESPONSE};
use crate::{
    app::error::KoboError,
    client::{prosa::Client, ProsaLocation},
};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde_json::Value;

pub async fn translate_get_state(
    client: &Client,
    book_id: &str,
    api_key: &str,
) -> Result<ReadingState, KoboError> {
    let state_response = client.fetch_state(&book_id, api_key)?;

    let status = match state_response.statistics.reading_status.as_ref() {
        "Read" => "Finished".to_string(),
        "Unread" => "ReadyToRead".to_string(),
        status => status.to_string(),
    };

    let state_location = state_response.location.as_ref();

    let state = ReadingState::new(
        &book_id,
        &status,
        state_location.and_then(|l| l.tag.clone()),
        state_location.and_then(|l| l.source.clone()),
    );

    Ok(state)
}

//TODO verify if having a status: unread book with active location causes bugs
//TODO it can indeed cause bugs
//TODO okay, maybe not, but it glitches when on titlepage and tag is kobo.1.1 (or doesn't exist in general)
//TODO probably revert prosa to reject only the tag if it is not present, but keep the source.
//TODO Probably best if we just add an exception to the title page. Or just verify if it even matters that the error happens at all
pub async fn translate_update_state(
    client: &Client,
    book_id: &str,
    state: &ReadingState,
    api_key: &str,
) -> Result<Value, KoboError> {
    //TODO remove
    println!("{:#?}", state);
    let mut previous_state = client.fetch_state(book_id, api_key)?;
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

    let tag = location.and_then(|l| Some(l.value.clone()));

    previous_state.statistics.reading_status = status;
    if tag.is_some() || source.is_some() {
        let loc = previous_state.location.get_or_insert(ProsaLocation {
            tag: None,
            source: None,
        });

        if let Some(s) = source {
            loc.source = Some(s);
        }
        if let Some(t) = tag.clone() {
            loc.tag = Some(t);
        }
    }

    if tag.unwrap_or_default() == "kobo.1.1"
        && (state.current_bookmark.progress_percent == 0 || state.current_bookmark.progress_percent == 100)
    {
        previous_state.location.as_mut().map(|l| l.tag = None);
    }

    client.update_state(&book_id, &previous_state, api_key)?;

    let response = &UPDATE_STATE_RESPONSE.replace("{book_id}", book_id);
    let response = serde_json::from_str(&response).expect("Failed to convert to JSON");
    Ok(response)
}

pub async fn translate_events(
    client: &Client,
    events: Value,
    api_key: &str,
) -> Result<EventResponse, KoboError> {
    let mut ids: Vec<String> = Vec::new();

    for event in events["Events"].as_array().expect("Object should be an array") {
        let id = event["Id"].as_str().expect("Id should be present");
        let event_type = event["EventType"].as_str().expect("EventType should be present");

        ids.push(id.to_string());

        if event_type != "RateBook" {
            continue;
        }

        let book_id = event["Attributes"]["volumeid"]
            .as_str()
            .expect("BookId should be present");
        let rating = event["Metrics"]["stars"]
            .as_u64()
            .expect("Rating should be present");
        let rating: u8 = rating.try_into().expect("Rating should be small");

        client.update_rating(book_id, rating, api_key)?;
    }

    Ok(EventResponse::new(ids))
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
