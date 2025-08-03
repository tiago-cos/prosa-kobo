use super::models::NewEntitlementResponse;
use crate::{
    app::{
        annotations,
        error::KoboError,
        metadata::{self, BookMetadata},
        shelves::models::{DeletedShelfResponse, NewShelfResponse},
        state::{self, models::ReadingState},
        sync::models::{BookEntitlement, SyncItem},
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
    book_expiration: i64,
    api_key: &str,
    device_id: &str,
) -> Result<Vec<SyncItem>, KoboError> {
    let sync_response = client.sync_device(since, api_key)?;

    let mut translated_response: Vec<SyncItem> = Vec::new();

    // Handle books

    let mut books_to_update: HashSet<String> = sync_response.book.file.into_iter().collect();
    books_to_update.extend(sync_response.book.cover);
    books_to_update.extend(sync_response.book.metadata);

    for book_id in books_to_update {
        let entitlement = BookEntitlement::new(&book_id, false);
        let reading_state = state::service::translate_get_state(client, &book_id, api_key).await?;
        let metadata = metadata::service::translate_metadata(
            pool,
            client,
            &book_id,
            server_url,
            book_expiration,
            api_key,
            device_id,
        )
        .await?;

        let response =
            SyncItem::Entitlement(NewEntitlementResponse::new(entitlement, reading_state, metadata));

        translated_response.push(response);
    }

    for book_id in sync_response.book.deleted {
        let entitlement = BookEntitlement::new(&book_id, true);
        let reading_state = ReadingState::default();
        let metadata = BookMetadata::default();
        let response =
            SyncItem::Entitlement(NewEntitlementResponse::new(entitlement, reading_state, metadata));

        translated_response.push(response);
    }

    // Handle annotations

    for book_id in sync_response.book.annotations {
        annotations::service::update_etag(pool, &book_id).await;
    }

    // Handle shelfs

    let mut shelfs_to_update: HashSet<String> = sync_response.shelf.metadata.into_iter().collect();
    shelfs_to_update.extend(sync_response.shelf.contents);

    for shelf_id in shelfs_to_update {
        let name = client.get_shelf_metadata(&shelf_id, api_key)?.name;
        let books = client.list_books_in_shelf(&shelf_id, api_key)?;
        let response = SyncItem::NewShelf(NewShelfResponse::new(&shelf_id, &name, &books));

        translated_response.push(response);
    }

    for shelf_id in sync_response.shelf.deleted {
        let response = SyncItem::DeletedShelf(DeletedShelfResponse::new(&shelf_id));

        translated_response.push(response);
    }

    Ok(translated_response)
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
