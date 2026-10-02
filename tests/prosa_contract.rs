mod common;

use async_trait::async_trait;
use common::fixture;
use prosa_kobo::client::{
    ProsaAnnotationRequest, ProsaReadingStatus,
    identity::{ProsaAuthType, ProsaIdentity},
    mock::MockProsaClient,
    prosa::{Client, ClientError, ProsaApi},
    state::{ProsaState, ProsaStatistics},
};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU32, Ordering};

const GATSBY: &str = "The_Great_Gatsby.epub";
const CHAPTER: &str = "OEBPS/7860148755851063127_64317-h-2.htm.xhtml";
const EVERY_CAPABILITY: [&str; 4] = ["Read", "Create", "Update", "Delete"];
const MISSING: &str = "00000000-0000-4000-8000-000000000000";

fn at(position: &str) -> String {
    format!("{CHAPTER}#{position}")
}

fn earlier() -> String {
    at("0/0/0/t0:0")
}

fn later() -> String {
    at("0/1/t0:2")
}

fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn unique(prefix: &str) -> String {
    format!("{prefix}{:08x}", rand::random::<u32>())
}

#[async_trait]
trait Backend: Sync {
    fn client(&self) -> &dyn ProsaApi;
    fn key(&self) -> &str;
    fn username(&self) -> &str;
    async fn add_book(&self) -> String;
}

struct Mock {
    client: MockProsaClient,
    books: AtomicU32,
}

const MOCK_KEY: &str = "a-mock-key";
const MOCK_USERNAME: &str = "reader";

impl Mock {
    fn new() -> Self {
        let client = MockProsaClient::new();
        client.seed_identity(
            MOCK_KEY,
            ProsaIdentity {
                auth_type: ProsaAuthType::ApiKey,
                user_id: "user".to_owned(),
                username: MOCK_USERNAME.to_owned(),
                is_admin: false,
                capabilities: EVERY_CAPABILITY.iter().map(|c| (*c).to_owned()).collect(),
                key_id: Some("key".to_owned()),
            },
        );

        Self {
            client,
            books: AtomicU32::new(0),
        }
    }
}

#[async_trait]
impl Backend for Mock {
    fn client(&self) -> &dyn ProsaApi {
        &self.client
    }

    fn key(&self) -> &str {
        MOCK_KEY
    }

    fn username(&self) -> &str {
        MOCK_USERNAME
    }

    async fn add_book(&self) -> String {
        let book_id = format!("book-{}", self.books.fetch_add(1, Ordering::Relaxed));

        self.client.seed_file(&book_id, fixture(GATSBY)).seed_state(
            &book_id,
            ProsaState {
                location: None,
                statistics: ProsaStatistics {
                    rating: None,
                    reading_status: ProsaReadingStatus::Unread,
                },
            },
        );

        book_id
    }
}

struct Live {
    client: Client,
    url: String,
    key: String,
    username: String,
}

impl Live {
    async fn new() -> Self {
        let url = std::env::var("PROSA_URL").unwrap_or_else(|_| "http://127.0.0.1:5000".to_owned());
        let (scheme, authority) = url.split_once("://").expect("PROSA_URL should name a scheme");
        let (host, port) = match authority.trim_end_matches('/').rsplit_once(':') {
            Some((host, port)) => (host, port.parse().expect("PROSA_URL should name a numeric port")),
            None => (authority, if scheme == "https" { 443 } else { 80 }),
        };

        let username = unique("contract-");
        let http = reqwest::Client::new();
        let mut register = http.post(format!("{url}/auth/register"));
        if let Ok(admin_key) = std::env::var("PROSA_ADMIN_KEY") {
            register = register.header("admin-key", admin_key);
        }
        let registered: Value = register
            .json(&json!({ "username": username, "password": "a-contract-password" }))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .expect("Failed to register a user on Prosa")
            .json()
            .await
            .expect("Registration should answer JSON");

        let user_id = registered["user_id"].as_str().expect("A user id");
        let jwt = registered["jwt_token"].as_str().expect("A token");
        let created: Value = http
            .post(format!("{url}/users/{user_id}/keys"))
            .header("Authorization", format!("Bearer {jwt}"))
            .json(&json!({ "name": "contract", "capabilities": EVERY_CAPABILITY }))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .expect("Failed to create an API key")
            .json()
            .await
            .expect("Key creation should answer JSON");

        Self {
            client: Client::new(scheme, host, port),
            key: created["key"].as_str().expect("A key").to_owned(),
            username,
            url,
        }
    }
}

#[async_trait]
impl Backend for Live {
    fn client(&self) -> &dyn ProsaApi {
        &self.client
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn username(&self) -> &str {
        &self.username
    }

    async fn add_book(&self) -> String {
        let boundary = "prosa-kobo-contract";
        let mut body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"epub\"; filename=\"{GATSBY}\"\r\n\
             Content-Type: application/epub+zip\r\n\r\n"
        )
        .into_bytes();
        body.extend(fixture(GATSBY));
        body.extend(format!("\r\n--{boundary}--\r\n").into_bytes());

        reqwest::Client::new()
            .post(format!("{}/books", self.url))
            .header("api-key", &self.key)
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .expect("Failed to upload a book to Prosa")
            .text()
            .await
            .expect("The upload should answer the book's id")
    }
}

fn annotation(start: &str, end: &str, annotation_id: Option<&str>) -> ProsaAnnotationRequest {
    ProsaAnnotationRequest {
        start_location: start.to_owned(),
        end_location: end.to_owned(),
        note: None,
        annotation_id: annotation_id.map(str::to_owned),
    }
}

async fn identifies_the_key_it_is_handed(backend: &dyn Backend) {
    let identity = backend
        .client()
        .identity(backend.key())
        .await
        .expect("The key should be recognized");

    assert_eq!(identity.auth_type, ProsaAuthType::ApiKey);
    assert_eq!(identity.username, backend.username());
    assert!(!identity.is_admin);
    assert!(identity.key_id.is_some());
    for capability in EVERY_CAPABILITY {
        assert!(identity.capabilities.iter().any(|held| held == capability));
    }

    assert_eq!(
        backend.client().identity("bm90IGEga2V5").await.err(),
        Some(ClientError::Unauthorized)
    );
}

async fn serves_back_the_book_it_holds_until_it_is_deleted(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;
    let epub = fixture(GATSBY);

    assert_eq!(client.download_book(&book, key).await.ok(), Some(epub.clone()));
    assert_eq!(
        client
            .fetch_book_file_metadata(&book, key)
            .await
            .map(|file| file.file_size)
            .ok(),
        Some(epub.len() as u64)
    );
    assert_eq!(
        client.download_book(MISSING, key).await.err(),
        Some(ClientError::NotFound)
    );

    assert_eq!(client.delete_book(&book, key).await, Ok(()));
    assert_eq!(
        client.download_book(&book, key).await.err(),
        Some(ClientError::NotFound)
    );
    assert_eq!(client.delete_book(&book, key).await, Err(ClientError::NotFound));
}

async fn has_no_cover_for_a_book_without_one(backend: &dyn Backend) {
    let book = backend.add_book().await;

    assert_eq!(
        backend.client().download_cover(&book, backend.key()).await.err(),
        Some(ClientError::NotFound)
    );
}

async fn keeps_a_reading_position_only_where_the_book_has_one(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;

    let fresh = client
        .fetch_state(&book, key)
        .await
        .expect("A book should have a state");
    assert_eq!(fresh.statistics.reading_status, ProsaReadingStatus::Unread);
    assert_eq!(fresh.location, None);

    assert_eq!(
        client
            .patch_state(&book, Some(&earlier()), ProsaReadingStatus::Reading, key)
            .await,
        Ok(())
    );
    let moved = client
        .fetch_state(&book, key)
        .await
        .expect("A book should have a state");
    assert_eq!(moved.location, Some(earlier()));
    assert_eq!(moved.statistics.reading_status, ProsaReadingStatus::Reading);

    assert_eq!(
        client
            .patch_state(&book, Some(&at("0/99/t0:0")), ProsaReadingStatus::Reading, key)
            .await,
        Err(ClientError::BadRequest)
    );
}

async fn replaces_a_state_as_long_as_it_is_valid(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;
    let rating = |state: Result<ProsaState, ClientError>| state.map(|state| state.statistics.rating);

    let mut state = client
        .fetch_state(&book, key)
        .await
        .expect("A book should have a state");

    state.statistics.rating = Some(4.0);
    assert_eq!(client.replace_state(&book, &state, key).await, Ok(()));
    assert_eq!(rating(client.fetch_state(&book, key).await), Ok(Some(4.0)));

    state.statistics.rating = None;
    assert_eq!(client.replace_state(&book, &state, key).await, Ok(()));
    assert_eq!(rating(client.fetch_state(&book, key).await), Ok(None));

    state.statistics.rating = Some(6.0);
    assert_eq!(
        client.replace_state(&book, &state, key).await,
        Err(ClientError::BadRequest)
    );

    state.statistics.rating = Some(4.0);
    state.location = Some(at("0/99/t0:0"));
    assert_eq!(
        client.replace_state(&book, &state, key).await,
        Err(ClientError::BadRequest)
    );
}

async fn holds_a_location_only_while_a_book_is_being_read(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;
    let fetch = || async {
        client
            .fetch_state(&book, key)
            .await
            .expect("A book should have a state")
    };

    assert_eq!(
        client
            .patch_state(&book, Some(&earlier()), ProsaReadingStatus::Unread, key)
            .await,
        Err(ClientError::BadRequest)
    );
    assert_eq!(
        client
            .patch_state(&book, Some(&earlier()), ProsaReadingStatus::Reading, key)
            .await,
        Ok(())
    );

    for status in [ProsaReadingStatus::Read, ProsaReadingStatus::Unread] {
        let mut state = fetch().await;
        state.statistics.reading_status = status;
        assert_eq!(
            client.replace_state(&book, &state, key).await,
            Err(ClientError::BadRequest)
        );
        assert_eq!(
            client.patch_state(&book, Some(&later()), status, key).await,
            Err(ClientError::BadRequest)
        );

        state.location = None;
        assert_eq!(client.replace_state(&book, &state, key).await, Ok(()));
        assert_eq!(fetch().await, state);

        state.location = Some(earlier());
        state.statistics.reading_status = ProsaReadingStatus::Reading;
        assert_eq!(client.replace_state(&book, &state, key).await, Ok(()));
    }
}

async fn drops_the_location_when_a_patch_puts_a_book_aside(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;

    for status in [ProsaReadingStatus::Read, ProsaReadingStatus::Unread] {
        let reading = ProsaState {
            location: Some(earlier()),
            statistics: ProsaStatistics {
                rating: Some(4.0),
                reading_status: ProsaReadingStatus::Reading,
            },
        };
        assert_eq!(client.replace_state(&book, &reading, key).await, Ok(()));

        assert_eq!(client.patch_state(&book, None, status, key).await, Ok(()));
        assert_eq!(
            client.fetch_state(&book, key).await,
            Ok(ProsaState {
                location: None,
                statistics: ProsaStatistics {
                    rating: Some(4.0),
                    reading_status: status,
                },
            })
        );
    }
}

async fn keeps_an_annotation_over_text_until_it_is_deleted(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;
    let id = uuid();

    assert_eq!(
        client
            .add_annotation(&book, &annotation(&earlier(), &later(), Some(&id)), key)
            .await,
        Ok(id.clone())
    );
    assert_eq!(client.list_annotations(&book, key).await, Ok(vec![id.clone()]));

    let stored = client
        .get_annotation(&book, &id, key)
        .await
        .expect("The annotation should be held");
    assert_eq!(stored.start_location, earlier());
    assert_eq!(stored.end_location, later());

    assert_eq!(client.patch_annotation(&book, &id, "A note", key).await, Ok(()));
    assert_eq!(
        client
            .get_annotation(&book, &id, key)
            .await
            .ok()
            .and_then(|stored| stored.note),
        Some("A note".to_owned())
    );

    assert_eq!(client.delete_annotation(&book, &id, key).await, Ok(()));
    assert_eq!(
        client.get_annotation(&book, &id, key).await.err(),
        Some(ClientError::NotFound)
    );
    assert_eq!(
        client.delete_annotation(&book, &id, key).await,
        Err(ClientError::NotFound)
    );
}

async fn refuses_an_annotation_that_is_not_a_stretch_of_text(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;

    for (start, end) in [
        (earlier(), earlier()),
        (earlier(), at("0/0")),
        (later(), earlier()),
    ] {
        assert_eq!(
            client
                .add_annotation(&book, &annotation(&start, &end, None), key)
                .await,
            Err(ClientError::BadRequest),
            "{start} .. {end} should be refused"
        );
    }
}

async fn refuses_to_annotate_a_span_or_reuse_an_id_twice(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;
    let id = uuid();

    client
        .add_annotation(&book, &annotation(&earlier(), &later(), Some(&id)), key)
        .await
        .expect("The first annotation should be taken");

    assert_eq!(
        client
            .add_annotation(&book, &annotation(&earlier(), &later(), Some(&uuid())), key)
            .await,
        Err(ClientError::Conflict)
    );
    assert_eq!(
        client
            .add_annotation(&book, &annotation(&earlier(), &at("0/1/t0:3"), Some(&id)), key)
            .await,
        Err(ClientError::Conflict)
    );
}

async fn refuses_an_annotation_id_that_is_not_a_uuid(backend: &dyn Backend) {
    let book = backend.add_book().await;

    assert_eq!(
        backend
            .client()
            .add_annotation(
                &book,
                &annotation(&earlier(), &later(), Some("not-a-uuid")),
                backend.key()
            )
            .await,
        Err(ClientError::BadRequest)
    );
}

async fn stores_an_annotation_id_in_its_canonical_form(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;
    let id = uuid();

    assert_eq!(
        client
            .add_annotation(
                &book,
                &annotation(&earlier(), &later(), Some(&id.to_uppercase())),
                key
            )
            .await,
        Ok(id.clone())
    );
    assert_eq!(client.list_annotations(&book, key).await, Ok(vec![id]));
}

async fn holds_a_shelf_name_once_per_user(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let name = unique("Shelf ");
    let renamed = unique("Renamed ");

    let shelf = client
        .create_shelf(&name, None, None, key)
        .await
        .expect("The shelf should be created");
    assert_eq!(
        client.create_shelf(&name, None, None, key).await,
        Err(ClientError::Conflict)
    );
    assert_eq!(
        client
            .get_shelf_metadata(&shelf, key)
            .await
            .map(|shelf| shelf.name),
        Ok(name)
    );

    assert_eq!(client.update_shelf_name(&shelf, &renamed, key).await, Ok(()));
    assert_eq!(
        client
            .get_shelf_metadata(&shelf, key)
            .await
            .map(|shelf| shelf.name),
        Ok(renamed)
    );

    assert_eq!(client.delete_shelf(&shelf, key).await, Ok(()));
    assert_eq!(client.delete_shelf(&shelf, key).await, Err(ClientError::NotFound));
}

async fn holds_each_book_on_a_shelf_once(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book().await;
    let shelf = client
        .create_shelf(&unique("Shelf "), None, None, key)
        .await
        .expect("The shelf should be created");

    assert_eq!(client.add_book_to_shelf(&shelf, &book, key).await, Ok(()));
    assert_eq!(
        client.add_book_to_shelf(&shelf, &book, key).await,
        Err(ClientError::Conflict)
    );
    assert_eq!(
        client.list_books_in_shelf(&shelf, key).await,
        Ok(vec![book.clone()])
    );

    assert_eq!(client.delete_book_from_shelf(&shelf, &book, key).await, Ok(()));
    assert_eq!(
        client.delete_book_from_shelf(&shelf, &book, key).await,
        Err(ClientError::NotFound)
    );
}

async fn refuses_to_shelve_a_book_it_does_not_hold(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let shelf = client
        .create_shelf(&unique("Shelf "), None, None, key)
        .await
        .expect("The shelf should be created");

    assert_eq!(
        client.add_book_to_shelf(&shelf, MISSING, key).await,
        Err(ClientError::NotFound)
    );
}

async fn finds_its_own_shelves_by_any_part_of_the_name(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let word = unique("word");
    let shelf = client
        .create_shelf(&format!("My {word} shelf"), None, None, key)
        .await
        .expect("The shelf should be created");

    assert_eq!(
        client
            .search_shelves(backend.username(), &word.to_uppercase(), key)
            .await,
        Ok(vec![shelf])
    );
    assert_eq!(
        client.search_shelves(&unique("nobody-"), &word, key).await,
        Err(ClientError::Forbidden)
    );
}

macro_rules! contract {
    ($($scenario:ident),* $(,)?) => {
        mod mock {
            $(
                #[tokio::test]
                async fn $scenario() {
                    super::$scenario(&super::Mock::new()).await;
                }
            )*
        }

        mod live {
            $(
                #[tokio::test]
                #[ignore = "needs a running Prosa at PROSA_URL"]
                async fn $scenario() {
                    super::$scenario(&super::Live::new().await).await;
                }
            )*
        }
    };
}

contract!(
    identifies_the_key_it_is_handed,
    serves_back_the_book_it_holds_until_it_is_deleted,
    has_no_cover_for_a_book_without_one,
    keeps_a_reading_position_only_where_the_book_has_one,
    replaces_a_state_as_long_as_it_is_valid,
    holds_a_location_only_while_a_book_is_being_read,
    drops_the_location_when_a_patch_puts_a_book_aside,
    keeps_an_annotation_over_text_until_it_is_deleted,
    refuses_an_annotation_that_is_not_a_stretch_of_text,
    refuses_to_annotate_a_span_or_reuse_an_id_twice,
    refuses_an_annotation_id_that_is_not_a_uuid,
    stores_an_annotation_id_in_its_canonical_form,
    holds_a_shelf_name_once_per_user,
    holds_each_book_on_a_shelf_once,
    refuses_to_shelve_a_book_it_does_not_hold,
    finds_its_own_shelves_by_any_part_of_the_name,
);
