use super::models::NewEntitlementResponse;
use crate::{
    app::{
        metadata::{self},
        sync::models::{BookEntitlement, ReadingState},
    },
    client::prosa::Client,
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

//TODO dont forget the last modified cookie
pub async fn translate_sync(
    pool: &SqlitePool,
    client: &Client,
    server_url: &str,
    download_expiration: i64,
    api_key: &str,
) -> Vec<NewEntitlementResponse> {
    let sync_response = client
        .sync_device(api_key)
        .expect("Sync response should not fail");

    let mut translated_response = Vec::new();

    for book_id in sync_response.file {
        let state_response = client
            .fetch_state(&book_id, api_key)
            .expect("State response should not fail");
        let state_location = state_response.location.as_ref();

        let entitlement = BookEntitlement::new(&book_id, false);
        let reading_state = ReadingState::new(
            &book_id,
            &state_response.statistics.reading_status,
            state_location.and_then(|l| l.tag.clone()),
            state_location.and_then(|l| l.source.clone()),
        );

        let metadata = metadata::service::translate_metadata(
            pool,
            client,
            &book_id,
            server_url,
            download_expiration,
            api_key,
        )
        .await;

        translated_response.push(NewEntitlementResponse::new(entitlement, reading_state, metadata));
    }

    translated_response
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
