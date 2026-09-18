//! An in-memory stand-in for a Prosa backend.
//!
//! It answers like the real server would -- writes are visible to later reads,
//! and a missing book or shelf is a `NotFound` -- while recording every call so
//! a test can assert what the middleware asked for. Any method can also be made
//! to fail on demand, which is how the error paths are reached without data.
//!
//! The crate has no library target, so outside its own tests nothing here is
//! called from within it.
#![cfg_attr(not(test), allow(dead_code))]

use super::{
    ProsaAnnotation, ProsaAnnotationRequest, ProsaMetadata, ProsaReadingStatus,
    book::ProsaBookFileMetadata,
    prosa::{ClientError, ProsaApi},
    shelf::ProsaShelfMetadata,
    state::ProsaState,
    sync::ProsaSync,
};
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

const POISONED: &str = "Mock lock poisoned";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProsaMethod {
    SyncDevice,
    FetchMetadata,
    FetchBookFileMetadata,
    DownloadBook,
    DeleteBook,
    DownloadCover,
    FetchState,
    PatchState,
    UpdateRating,
    FetchRating,
    ListAnnotations,
    GetAnnotation,
    AddAnnotation,
    PatchAnnotation,
    DeleteAnnotation,
    CreateShelf,
    GetShelfMetadata,
    UpdateShelfName,
    DeleteShelf,
    AddBookToShelf,
    ListBooksInShelf,
    DeleteBookFromShelf,
}

/// One recorded request: the method, the identifiers it addressed (in argument
/// order) and the key it was made with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProsaCall {
    pub method: ProsaMethod,
    pub arguments: Vec<String>,
    pub api_key: String,
}

#[derive(Clone, Default, Debug)]
struct MockBook {
    file: Option<Vec<u8>>,
    file_metadata: Option<ProsaBookFileMetadata>,
    metadata: Option<ProsaMetadata>,
    cover: Option<Vec<u8>>,
    state: Option<ProsaState>,
    annotations: Vec<ProsaAnnotation>,
}

#[derive(Clone, Debug)]
struct MockShelf {
    name: String,
    owner_id: String,
    books: Vec<String>,
}

#[derive(Default)]
struct MockLibrary {
    books: HashMap<String, MockBook>,
    shelves: HashMap<String, MockShelf>,
    sync: ProsaSync,
    next_id: u32,
}

impl MockLibrary {
    fn book(&mut self, book_id: &str) -> Result<&mut MockBook, ClientError> {
        self.books.get_mut(book_id).ok_or(ClientError::NotFound)
    }

    fn shelf(&mut self, shelf_id: &str) -> Result<&mut MockShelf, ClientError> {
        self.shelves.get_mut(shelf_id).ok_or(ClientError::NotFound)
    }

    fn generate_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}-{}", self.next_id)
    }
}

#[derive(Default)]
pub struct MockProsaClient {
    library: Mutex<MockLibrary>,
    errors: Mutex<HashMap<ProsaMethod, ClientError>>,
    calls: Mutex<Vec<ProsaCall>>,
}

impl MockProsaClient {
    pub fn new() -> Self {
        Self::default()
    }

    // Seeding

    /// Registers a book with no data attached, so calls addressing it stop
    /// answering `NotFound`.
    pub fn seed_book(&self, book_id: &str) -> &Self {
        self.library().books.entry(book_id.to_owned()).or_default();
        self
    }

    pub fn seed_file(&self, book_id: &str, file: Vec<u8>) -> &Self {
        let file_metadata = ProsaBookFileMetadata {
            owner_id: "owner".to_owned(),
            file_size: file.len() as u64,
        };

        let mut library = self.library();
        let book = library.books.entry(book_id.to_owned()).or_default();
        book.file = Some(file);
        book.file_metadata.get_or_insert(file_metadata);

        drop(library);
        self
    }

    pub fn seed_file_metadata(&self, book_id: &str, file_metadata: ProsaBookFileMetadata) -> &Self {
        self.library()
            .books
            .entry(book_id.to_owned())
            .or_default()
            .file_metadata = Some(file_metadata);

        self
    }

    pub fn seed_metadata(&self, book_id: &str, metadata: ProsaMetadata) -> &Self {
        self.library()
            .books
            .entry(book_id.to_owned())
            .or_default()
            .metadata = Some(metadata);

        self
    }

    pub fn seed_cover(&self, book_id: &str, cover: Vec<u8>) -> &Self {
        self.library().books.entry(book_id.to_owned()).or_default().cover = Some(cover);

        self
    }

    pub fn seed_state(&self, book_id: &str, state: ProsaState) -> &Self {
        self.library().books.entry(book_id.to_owned()).or_default().state = Some(state);

        self
    }

    pub fn seed_annotation(&self, book_id: &str, annotation: ProsaAnnotation) -> &Self {
        self.library()
            .books
            .entry(book_id.to_owned())
            .or_default()
            .annotations
            .push(annotation);

        self
    }

    pub fn seed_shelf(&self, shelf_id: &str, name: &str, book_ids: &[&str]) -> &Self {
        self.library().shelves.insert(
            shelf_id.to_owned(),
            MockShelf {
                name: name.to_owned(),
                owner_id: "owner".to_owned(),
                books: book_ids.iter().map(|id| (*id).to_owned()).collect(),
            },
        );

        self
    }

    pub fn seed_sync(&self, sync: ProsaSync) -> &Self {
        self.library().sync = sync;
        self
    }

    /// Makes every later call to `method` fail, until [`Self::succeed`] clears it.
    pub fn fail(&self, method: ProsaMethod, error: ClientError) -> &Self {
        self.errors.lock().expect(POISONED).insert(method, error);
        self
    }

    pub fn succeed(&self, method: ProsaMethod) -> &Self {
        self.errors.lock().expect(POISONED).remove(&method);
        self
    }

    // Inspection

    pub fn calls(&self) -> Vec<ProsaCall> {
        self.calls.lock().expect(POISONED).clone()
    }

    pub fn calls_to(&self, method: ProsaMethod) -> Vec<ProsaCall> {
        self.calls()
            .into_iter()
            .filter(|call| call.method == method)
            .collect()
    }

    pub fn call_count(&self, method: ProsaMethod) -> usize {
        self.calls_to(method).len()
    }

    pub fn stored_state(&self, book_id: &str) -> Option<ProsaState> {
        self.library().books.get(book_id)?.state.clone()
    }

    pub fn stored_annotations(&self, book_id: &str) -> Vec<ProsaAnnotation> {
        self.library()
            .books
            .get(book_id)
            .map(|book| book.annotations.clone())
            .unwrap_or_default()
    }

    pub fn stored_shelf_books(&self, shelf_id: &str) -> Vec<String> {
        self.library()
            .shelves
            .get(shelf_id)
            .map(|shelf| shelf.books.clone())
            .unwrap_or_default()
    }

    pub fn stored_shelf_name(&self, shelf_id: &str) -> Option<String> {
        Some(self.library().shelves.get(shelf_id)?.name.clone())
    }

    // Internals

    fn library(&self) -> MutexGuard<'_, MockLibrary> {
        self.library.lock().expect(POISONED)
    }

    fn record(&self, method: ProsaMethod, arguments: &[&str], api_key: &str) -> Result<(), ClientError> {
        let call = ProsaCall {
            method,
            arguments: arguments.iter().map(|argument| (*argument).to_owned()).collect(),
            api_key: api_key.to_owned(),
        };

        self.calls.lock().expect(POISONED).push(call);

        match self.errors.lock().expect(POISONED).get(&method) {
            Some(error) => Err(*error),
            None => Ok(()),
        }
    }
}

impl ProsaApi for MockProsaClient {
    fn sync_device(&self, sync_token: Option<i64>, api_key: &str) -> Result<ProsaSync, ClientError> {
        let token = sync_token.map(|token| token.to_string()).unwrap_or_default();
        self.record(ProsaMethod::SyncDevice, &[&token], api_key)?;

        Ok(self.library().sync.clone())
    }

    fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, ClientError> {
        self.record(ProsaMethod::FetchMetadata, &[book_id], api_key)?;

        self.library()
            .book(book_id)?
            .metadata
            .clone()
            .ok_or(ClientError::NotFound)
    }

    fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, ClientError> {
        self.record(ProsaMethod::FetchBookFileMetadata, &[book_id], api_key)?;

        self.library()
            .book(book_id)?
            .file_metadata
            .clone()
            .ok_or(ClientError::NotFound)
    }

    fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        self.record(ProsaMethod::DownloadBook, &[book_id], api_key)?;

        self.library()
            .book(book_id)?
            .file
            .clone()
            .ok_or(ClientError::NotFound)
    }

    fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.record(ProsaMethod::DeleteBook, &[book_id], api_key)?;

        self.library()
            .books
            .remove(book_id)
            .map(|_| ())
            .ok_or(ClientError::NotFound)
    }

    fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        self.record(ProsaMethod::DownloadCover, &[book_id], api_key)?;

        self.library()
            .book(book_id)?
            .cover
            .clone()
            .ok_or(ClientError::NotFound)
    }

    fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, ClientError> {
        self.record(ProsaMethod::FetchState, &[book_id], api_key)?;

        self.library()
            .book(book_id)?
            .state
            .clone()
            .ok_or(ClientError::NotFound)
    }

    fn patch_state(
        &self,
        book_id: &str,
        location: Option<&str>,
        reading_status: ProsaReadingStatus,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.record(
            ProsaMethod::PatchState,
            &[book_id, location.unwrap_or_default()],
            api_key,
        )?;

        let mut library = self.library();
        let state = library
            .book(book_id)?
            .state
            .as_mut()
            .ok_or(ClientError::NotFound)?;

        if let Some(location) = location {
            state.location = Some(location.to_owned());
        }

        state.statistics.reading_status = reading_status;

        Ok(())
    }

    fn update_rating(&self, book_id: &str, rating: u8, api_key: &str) -> Result<(), ClientError> {
        self.record(
            ProsaMethod::UpdateRating,
            &[book_id, &rating.to_string()],
            api_key,
        )?;

        let mut library = self.library();
        let state = library
            .book(book_id)?
            .state
            .as_mut()
            .ok_or(ClientError::NotFound)?;

        state.statistics.rating = match rating {
            0 => None,
            rating => Some(rating.into()),
        };

        Ok(())
    }

    fn fetch_rating(&self, book_id: &str, api_key: &str) -> Result<Option<u8>, ClientError> {
        self.record(ProsaMethod::FetchRating, &[book_id], api_key)?;

        let mut library = self.library();
        let state = library
            .book(book_id)?
            .state
            .as_ref()
            .ok_or(ClientError::NotFound)?;

        Ok(state
            .statistics
            .rating
            .map(|rating| rating.round().clamp(0.0, 255.0) as u8))
    }

    fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        self.record(ProsaMethod::ListAnnotations, &[book_id], api_key)?;

        let annotations = self
            .library()
            .book(book_id)?
            .annotations
            .iter()
            .map(|annotation| annotation.annotation_id.clone())
            .collect();

        Ok(annotations)
    }

    fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, ClientError> {
        self.record(ProsaMethod::GetAnnotation, &[book_id, annotation_id], api_key)?;

        self.library()
            .book(book_id)?
            .annotations
            .iter()
            .find(|annotation| annotation.annotation_id == annotation_id)
            .cloned()
            .ok_or(ClientError::NotFound)
    }

    fn add_annotation(
        &self,
        book_id: &str,
        annotation: &ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, ClientError> {
        self.record(
            ProsaMethod::AddAnnotation,
            &[book_id, annotation.annotation_id.as_deref().unwrap_or_default()],
            api_key,
        )?;

        let mut library = self.library();
        let annotation_id = match &annotation.annotation_id {
            Some(annotation_id) => annotation_id.clone(),
            None => library.generate_id("annotation"),
        };

        let book = library.book(book_id)?;

        if book
            .annotations
            .iter()
            .any(|stored| stored.annotation_id == annotation_id)
        {
            return Err(ClientError::Conflict);
        }

        book.annotations.push(ProsaAnnotation {
            annotation_id: annotation_id.clone(),
            start_location: annotation.start_location.clone(),
            end_location: annotation.end_location.clone(),
            note: annotation.note.clone(),
        });

        Ok(annotation_id)
    }

    fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.record(ProsaMethod::PatchAnnotation, &[book_id, annotation_id], api_key)?;

        let mut library = self.library();
        let annotation = library
            .book(book_id)?
            .annotations
            .iter_mut()
            .find(|annotation| annotation.annotation_id == annotation_id)
            .ok_or(ClientError::NotFound)?;

        annotation.note = match note {
            "" => None,
            note => Some(note.to_owned()),
        };

        Ok(())
    }

    fn delete_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.record(ProsaMethod::DeleteAnnotation, &[book_id, annotation_id], api_key)?;

        let mut library = self.library();
        let annotations = &mut library.book(book_id)?.annotations;
        let before = annotations.len();

        annotations.retain(|annotation| annotation.annotation_id != annotation_id);

        if annotations.len() == before {
            return Err(ClientError::NotFound);
        }

        Ok(())
    }

    fn create_shelf(
        &self,
        shelf_name: &str,
        owner_id: Option<&str>,
        shelf_id: Option<&str>,
        api_key: &str,
    ) -> Result<String, ClientError> {
        self.record(
            ProsaMethod::CreateShelf,
            &[shelf_name, shelf_id.unwrap_or_default()],
            api_key,
        )?;

        let mut library = self.library();

        if library.shelves.values().any(|shelf| shelf.name == shelf_name) {
            return Err(ClientError::Conflict);
        }

        let shelf_id = match shelf_id {
            Some(shelf_id) => shelf_id.to_owned(),
            None => library.generate_id("shelf"),
        };

        if library.shelves.contains_key(&shelf_id) {
            return Err(ClientError::Conflict);
        }

        library.shelves.insert(
            shelf_id.clone(),
            MockShelf {
                name: shelf_name.to_owned(),
                owner_id: owner_id.unwrap_or("owner").to_owned(),
                books: Vec::new(),
            },
        );

        Ok(shelf_id)
    }

    fn get_shelf_metadata(&self, shelf_id: &str, api_key: &str) -> Result<ProsaShelfMetadata, ClientError> {
        self.record(ProsaMethod::GetShelfMetadata, &[shelf_id], api_key)?;

        let mut library = self.library();
        let stored = library.shelf(shelf_id)?;

        Ok(ProsaShelfMetadata {
            name: stored.name.clone(),
            owner_id: stored.owner_id.clone(),
            book_count: stored.books.len() as u64,
        })
    }

    fn update_shelf_name(&self, shelf_id: &str, shelf_name: &str, api_key: &str) -> Result<(), ClientError> {
        self.record(ProsaMethod::UpdateShelfName, &[shelf_id, shelf_name], api_key)?;

        shelf_name.clone_into(&mut self.library().shelf(shelf_id)?.name);

        Ok(())
    }

    fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.record(ProsaMethod::DeleteShelf, &[shelf_id], api_key)?;

        self.library()
            .shelves
            .remove(shelf_id)
            .map(|_| ())
            .ok_or(ClientError::NotFound)
    }

    fn add_book_to_shelf(&self, shelf_id: &str, book_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.record(ProsaMethod::AddBookToShelf, &[shelf_id, book_id], api_key)?;

        let mut library = self.library();
        let books = &mut library.shelf(shelf_id)?.books;

        if books.iter().any(|stored| stored == book_id) {
            return Err(ClientError::Conflict);
        }

        books.push(book_id.to_owned());

        Ok(())
    }

    fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        self.record(ProsaMethod::ListBooksInShelf, &[shelf_id], api_key)?;

        Ok(self.library().shelf(shelf_id)?.books.clone())
    }

    fn delete_book_from_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.record(ProsaMethod::DeleteBookFromShelf, &[shelf_id, book_id], api_key)?;

        let mut library = self.library();
        let books = &mut library.shelf(shelf_id)?.books;
        let before = books.len();

        books.retain(|stored| stored != book_id);

        if books.len() == before {
            return Err(ClientError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{state::ProsaStatistics, sync::ProsaBookSync};

    fn annotation(annotation_id: &str) -> ProsaAnnotationRequest {
        ProsaAnnotationRequest {
            start_location: "OEBPS/chapter-001.xhtml#0/2/t1:44".to_owned(),
            end_location: "OEBPS/chapter-001.xhtml#0/3/t0:12".to_owned(),
            note: None,
            annotation_id: Some(annotation_id.to_owned()),
        }
    }

    #[test]
    fn reads_back_what_was_written() {
        let client = MockProsaClient::new();
        client.seed_book("book");

        client
            .add_annotation("book", &annotation("annotation"), "key")
            .expect("Failed to add annotation");

        let listed = client
            .list_annotations("book", "key")
            .expect("Failed to list annotations");

        assert_eq!(listed, vec!["annotation".to_owned()]);
    }

    #[test]
    fn refuses_an_annotation_id_already_in_use() {
        let client = MockProsaClient::new();
        client.seed_book("book");

        client
            .add_annotation("book", &annotation("annotation"), "key")
            .expect("Failed to add annotation");

        let conflict = client.add_annotation("book", &annotation("annotation"), "key");

        assert_eq!(conflict, Err(ClientError::Conflict));
    }

    #[test]
    fn answers_not_found_for_an_unknown_book() {
        let client = MockProsaClient::new();

        assert_eq!(client.fetch_state("book", "key"), Err(ClientError::NotFound));
    }

    #[test]
    fn records_every_call_with_its_key() {
        let client = MockProsaClient::new();
        let _ = client.fetch_metadata("book", "key");

        assert_eq!(
            client.calls_to(ProsaMethod::FetchMetadata),
            vec![ProsaCall {
                method: ProsaMethod::FetchMetadata,
                arguments: vec!["book".to_owned()],
                api_key: "key".to_owned(),
            }]
        );
    }

    #[test]
    fn serves_the_book_data_it_was_seeded_with() {
        let client = MockProsaClient::new();
        client.seed_file("book", vec![1, 2, 3]);
        client.seed_cover("book", vec![4, 5]);
        client.seed_metadata(
            "book",
            ProsaMetadata {
                title: Some("A Book".to_owned()),
                ..ProsaMetadata::default()
            },
        );

        assert_eq!(client.download_book("book", "key"), Ok(vec![1, 2, 3]));
        assert_eq!(client.download_cover("book", "key"), Ok(vec![4, 5]));

        let file_metadata = client
            .fetch_book_file_metadata("book", "key")
            .expect("Failed to fetch file metadata");

        assert_eq!(file_metadata.file_size, 3);

        let metadata = client
            .fetch_metadata("book", "key")
            .expect("Failed to fetch metadata");

        assert_eq!(metadata.title.as_deref(), Some("A Book"));
    }

    #[test]
    fn prefers_explicitly_seeded_file_metadata() {
        let client = MockProsaClient::new();
        client.seed_file("book", vec![1, 2, 3]);
        client.seed_file_metadata(
            "book",
            ProsaBookFileMetadata {
                owner_id: "someone".to_owned(),
                file_size: 99,
            },
        );

        let file_metadata = client
            .fetch_book_file_metadata("book", "key")
            .expect("Failed to fetch file metadata");

        assert_eq!(file_metadata.owner_id, "someone");
        assert_eq!(file_metadata.file_size, 99);
    }

    #[test]
    fn keeps_the_reading_state_it_is_handed() {
        let client = MockProsaClient::new();
        client.seed_state(
            "book",
            ProsaState {
                location: None,
                statistics: ProsaStatistics {
                    rating: None,
                    reading_status: ProsaReadingStatus::Unread,
                },
            },
        );

        client
            .patch_state(
                "book",
                Some("OEBPS/chapter-001.xhtml#0/2/t1:44"),
                ProsaReadingStatus::Reading,
                "key",
            )
            .expect("Failed to patch state");

        client
            .update_rating("book", 4, "key")
            .expect("Failed to update rating");

        let stored = client.stored_state("book").expect("Expected a stored state");

        assert_eq!(
            stored.location.as_deref(),
            Some("OEBPS/chapter-001.xhtml#0/2/t1:44")
        );
        assert_eq!(stored.statistics.reading_status, ProsaReadingStatus::Reading);
        assert_eq!(client.fetch_rating("book", "key"), Ok(Some(4)));
    }

    #[test]
    fn hands_back_the_seeded_sync_response() {
        let client = MockProsaClient::new();
        client.seed_sync(ProsaSync {
            new_sync_token: 42,
            unsynced_books: ProsaBookSync {
                file: vec!["book".to_owned()],
                ..ProsaBookSync::default()
            },
            ..ProsaSync::default()
        });

        let sync = client.sync_device(Some(7), "key").expect("Failed to sync");

        assert_eq!(sync.new_sync_token, 42);
        assert_eq!(sync.unsynced_books.file, vec!["book".to_owned()]);
        assert_eq!(
            client.calls_to(ProsaMethod::SyncDevice)[0].arguments,
            vec!["7".to_owned()]
        );
    }

    #[test]
    fn renames_a_shelf_in_place() {
        let client = MockProsaClient::new();
        client.seed_shelf("shelf", "Favourites", &[]);

        client
            .update_shelf_name("shelf", "Sci-Fi", "key")
            .expect("Failed to rename shelf");

        assert_eq!(client.stored_shelf_name("shelf").as_deref(), Some("Sci-Fi"));
    }

    #[test]
    fn fails_on_demand_until_cleared() {
        let client = MockProsaClient::new();
        client.seed_shelf("shelf", "Favourites", &[]);
        client.fail(ProsaMethod::GetShelfMetadata, ClientError::Forbidden);

        assert_eq!(
            client.get_shelf_metadata("shelf", "key"),
            Err(ClientError::Forbidden)
        );

        client.succeed(ProsaMethod::GetShelfMetadata);

        let metadata = client
            .get_shelf_metadata("shelf", "key")
            .expect("Failed to fetch shelf metadata");

        assert_eq!(metadata.name, "Favourites");
    }
}
