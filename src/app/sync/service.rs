use super::models::NewEntitlementResponse;
use crate::{
    app::{
        annotations,
        metadata::{self, BookMetadata},
        state::{self, models::ReadingState},
        sync::models::BookEntitlement,
    },
    client::prosa::Client,
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

//TODO see why the publisher isn't being sent when adding the ember in the ashes book
pub async fn translate_sync(
    pool: &SqlitePool,
    client: &Client,
    since: Option<i64>,
    server_url: &str,
    download_expiration: i64,
    api_key: &str,
) -> Vec<NewEntitlementResponse> {
    let sync_response = client
        .sync_device(since, api_key)
        .expect("Sync response should not fail");

    let mut translated_response = Vec::new();

    let mut books_to_update: HashSet<String> = sync_response.file.into_iter().collect();
    books_to_update.extend(sync_response.cover);
    books_to_update.extend(sync_response.metadata);

    for book_id in books_to_update {
        let entitlement = BookEntitlement::new(&book_id, false);
        let reading_state = state::service::translate_get_state(client, &book_id, api_key).await;
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

    for book_id in sync_response.deleted {
        let entitlement = BookEntitlement::new(&book_id, true);
        let reading_state = ReadingState::default();
        let metadata = BookMetadata::default();
        let response = NewEntitlementResponse::new(entitlement, reading_state, metadata);

        translated_response.push(response);
    }

    for book_id in sync_response.annotations {
        annotations::service::update_etag(pool, &book_id).await;
    }

    translated_response
}

pub async fn create_new_sync_token() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    now.to_string()
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
