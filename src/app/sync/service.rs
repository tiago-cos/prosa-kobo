use super::models::NewEntitlementResponse;
use crate::{
    app::{
        annotations, covers,
        error::KoboError,
        metadata::{self, BookMetadata},
        shelves::models::{DeletedShelfResponse, NewShelfResponse},
        state::{self, models::ReadingState},
        sync::models::{BookEntitlement, SyncItem},
    },
    client::prosa::ProsaApi,
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::collections::HashSet;

/// Returns the sync token the device should present next, alongside the
/// changes it has to apply.
pub async fn translate_sync(
    pool: &SqlitePool,
    client: &dyn ProsaApi,
    sync_token: Option<i64>,
    server_url: &str,
    book_expiration: i64,
    api_key: &str,
    device_id: &str,
) -> Result<(i64, Vec<SyncItem>), KoboError> {
    let sync_response = client.sync_device(sync_token, api_key)?;
    let new_sync_token = sync_response.new_sync_token;
    let books = sync_response.unsynced_books;
    let shelves = sync_response.unsynced_shelves;

    let mut translated_response: Vec<SyncItem> = Vec::new();

    // Handle books

    for book_id in &books.cover {
        covers::update_token(pool, book_id, device_id).await;
    }

    let mut books_to_update: HashSet<String> = books.file.into_iter().collect();
    books_to_update.extend(books.cover);
    books_to_update.extend(books.metadata);

    for book_id in books_to_update {
        let entitlement = BookEntitlement::new(&book_id, false);
        let reading_state = state::service::translate_get_state(client, &book_id, api_key)?;
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

    for book_id in books.deleted {
        let entitlement = BookEntitlement::new(&book_id, true);
        let reading_state = ReadingState::default();
        let metadata = BookMetadata::default();
        let response =
            SyncItem::Entitlement(NewEntitlementResponse::new(entitlement, reading_state, metadata));

        translated_response.push(response);
    }

    // Handle annotations

    for book_id in books.annotations {
        annotations::service::update_etag(pool, &book_id).await;
    }

    // Handle shelfs

    let mut shelfs_to_update: HashSet<String> = shelves.metadata.into_iter().collect();
    shelfs_to_update.extend(shelves.contents);

    for shelf_id in shelfs_to_update {
        let name = client.get_shelf_metadata(&shelf_id, api_key)?.name;
        let books = client.list_books_in_shelf(&shelf_id, api_key)?;
        let response = SyncItem::NewShelf(NewShelfResponse::new(&shelf_id, &name, &books));

        translated_response.push(response);
    }

    for shelf_id in shelves.deleted {
        let response = SyncItem::DeletedShelf(DeletedShelfResponse::new(&shelf_id));

        translated_response.push(response);
    }

    Ok((new_sync_token, translated_response))
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
