use super::models::{ChangedReadingStateResponse, NewEntitlementResponse};
use crate::{
    app::{
        annotations, covers,
        error::KoboError,
        kepub,
        metadata::{self, BookMetadata},
        shelves::models::{DeletedShelfResponse, NewShelfResponse},
        state::{self, models::ReadingState},
        sync::models::{BookEntitlement, SyncItem},
    },
    client::prosa::ProsaApi,
};
use std::collections::HashSet;

pub async fn translate_sync(
    client: &dyn ProsaApi,
    sync_token: Option<i64>,
    server_url: &str,
    api_key: &str,
    device_id: &str,
) -> Result<(i64, Vec<SyncItem>), KoboError> {
    let sync_response = client.sync_device(sync_token, api_key).await?;
    let new_sync_token = sync_response.new_sync_token;
    let books = sync_response.unsynced_books;
    let shelves = sync_response.unsynced_shelves;

    let mut translated_response: Vec<SyncItem> = Vec::new();

    // Handle books

    for book_id in &books.cover {
        covers::bump_version(device_id, book_id).await?;
    }

    for book_id in &books.file {
        kepub::evict(book_id);
    }

    let mut books_to_update: HashSet<String> = books.file.into_iter().collect();
    books_to_update.extend(books.cover);
    books_to_update.extend(books.metadata);

    let states_to_update: HashSet<String> = books
        .state
        .into_iter()
        .filter(|book_id| !books_to_update.contains(book_id) && !books.deleted.contains(book_id))
        .collect();

    for book_id in books_to_update {
        let entitlement = BookEntitlement::new(&book_id, false);
        let reading_state = state::service::translate_get_state(client, &book_id, api_key).await?;
        let metadata =
            metadata::service::translate_metadata(client, &book_id, server_url, api_key, device_id).await?;

        let response =
            SyncItem::Entitlement(NewEntitlementResponse::new(entitlement, reading_state, metadata));

        translated_response.push(response);
    }

    for book_id in states_to_update {
        let reading_state = state::service::translate_get_state(client, &book_id, api_key).await?;
        let response = SyncItem::ReadingState(ChangedReadingStateResponse::new(reading_state));

        translated_response.push(response);
    }

    for book_id in books.deleted {
        let entitlement = BookEntitlement::new(&book_id, true);
        let reading_state = ReadingState::for_removed_book();
        let metadata = BookMetadata::for_removed_book();
        let response =
            SyncItem::Entitlement(NewEntitlementResponse::new(entitlement, reading_state, metadata));

        translated_response.push(response);
    }

    // Handle annotations

    for book_id in books.annotations {
        annotations::service::update_etag(&book_id).await?;
    }

    // Handle shelfs

    let mut shelfs_to_update: HashSet<String> = shelves.metadata.into_iter().collect();
    shelfs_to_update.extend(shelves.contents);

    for shelf_id in shelfs_to_update {
        let name = client.get_shelf_metadata(&shelf_id, api_key).await?.name;
        let books = client.list_books_in_shelf(&shelf_id, api_key).await?;
        let response = SyncItem::NewShelf(NewShelfResponse::new(&shelf_id, &name, &books));

        translated_response.push(response);
    }

    for shelf_id in shelves.deleted {
        let response = SyncItem::DeletedShelf(DeletedShelfResponse::new(&shelf_id));

        translated_response.push(response);
    }

    Ok((new_sync_token, translated_response))
}
