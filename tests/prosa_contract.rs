mod common;

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

trait Backend {
    fn client(&self) -> &dyn ProsaApi;
    fn key(&self) -> &str;
    fn username(&self) -> &str;
    fn add_book(&self) -> String;
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

    fn add_book(&self) -> String {
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
    fn new() -> Self {
        let url = std::env::var("PROSA_URL").unwrap_or_else(|_| "http://127.0.0.1:5000".to_owned());
        let (scheme, authority) = url.split_once("://").expect("PROSA_URL should name a scheme");
        let (host, port) = match authority.trim_end_matches('/').rsplit_once(':') {
            Some((host, port)) => (host, port.parse().expect("PROSA_URL should name a numeric port")),
            None => (authority, if scheme == "https" { 443 } else { 80 }),
        };

        let username = unique("contract-");
        let mut register = ureq::post(format!("{url}/auth/register"));
        if let Ok(admin_key) = std::env::var("PROSA_ADMIN_KEY") {
            register = register.header("admin-key", admin_key);
        }
        let registered: Value = register
            .send_json(json!({ "username": username, "password": "a-contract-password" }))
            .expect("Failed to register a user on Prosa")
            .body_mut()
            .read_json()
            .expect("Registration should answer JSON");

        let user_id = registered["user_id"].as_str().expect("A user id");
        let jwt = registered["jwt_token"].as_str().expect("A token");
        let created: Value = ureq::post(format!("{url}/users/{user_id}/keys"))
            .header("Authorization", format!("Bearer {jwt}"))
            .send_json(json!({ "name": "contract", "capabilities": EVERY_CAPABILITY }))
            .expect("Failed to create an API key")
            .body_mut()
            .read_json()
            .expect("Key creation should answer JSON");

        Self {
            client: Client::new(scheme, host, port),
            key: created["key"].as_str().expect("A key").to_owned(),
            username,
            url,
        }
    }
}

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

    fn add_book(&self) -> String {
        let boundary = "prosa-kobo-contract";
        let mut body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"epub\"; filename=\"{GATSBY}\"\r\n\
             Content-Type: application/epub+zip\r\n\r\n"
        )
        .into_bytes();
        body.extend(fixture(GATSBY));
        body.extend(format!("\r\n--{boundary}--\r\n").into_bytes());

        ureq::post(format!("{}/books", self.url))
            .header("api-key", &self.key)
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .send(&body[..])
            .expect("Failed to upload a book to Prosa")
            .body_mut()
            .read_to_string()
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

fn identifies_the_key_it_is_handed(backend: &dyn Backend) {
    let identity = backend
        .client()
        .identity(backend.key())
        .expect("The key should be recognized");

    assert_eq!(identity.auth_type, ProsaAuthType::ApiKey);
    assert_eq!(identity.username, backend.username());
    assert!(!identity.is_admin);
    assert!(identity.key_id.is_some());
    for capability in EVERY_CAPABILITY {
        assert!(identity.capabilities.iter().any(|held| held == capability));
    }

    assert_eq!(
        backend.client().identity("bm90IGEga2V5").err(),
        Some(ClientError::Unauthorized)
    );
}

fn serves_back_the_book_it_holds_until_it_is_deleted(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book();
    let epub = fixture(GATSBY);

    assert_eq!(client.download_book(&book, key).ok(), Some(epub.clone()));
    assert_eq!(
        client
            .fetch_book_file_metadata(&book, key)
            .map(|file| file.file_size)
            .ok(),
        Some(epub.len() as u64)
    );
    assert_eq!(
        client.download_book(MISSING, key).err(),
        Some(ClientError::NotFound)
    );

    assert_eq!(client.delete_book(&book, key), Ok(()));
    assert_eq!(
        client.download_book(&book, key).err(),
        Some(ClientError::NotFound)
    );
    assert_eq!(client.delete_book(&book, key), Err(ClientError::NotFound));
}

fn has_no_cover_for_a_book_without_one(backend: &dyn Backend) {
    let book = backend.add_book();

    assert_eq!(
        backend.client().download_cover(&book, backend.key()).err(),
        Some(ClientError::NotFound)
    );
}

fn keeps_a_reading_position_only_where_the_book_has_one(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book();

    let fresh = client
        .fetch_state(&book, key)
        .expect("A book should have a state");
    assert_eq!(fresh.statistics.reading_status, ProsaReadingStatus::Unread);
    assert_eq!(fresh.location, None);

    assert_eq!(
        client.patch_state(&book, Some(&earlier()), ProsaReadingStatus::Reading, key),
        Ok(())
    );
    let moved = client
        .fetch_state(&book, key)
        .expect("A book should have a state");
    assert_eq!(moved.location, Some(earlier()));
    assert_eq!(moved.statistics.reading_status, ProsaReadingStatus::Reading);

    assert_eq!(
        client.patch_state(&book, Some(&at("0/99/t0:0")), ProsaReadingStatus::Reading, key),
        Err(ClientError::BadRequest)
    );
}

fn keeps_a_rating_and_clears_it_at_zero(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book();

    assert_eq!(client.update_rating(&book, 4, key), Ok(()));
    assert_eq!(client.fetch_rating(&book, key), Ok(Some(4)));

    assert_eq!(client.update_rating(&book, 0, key), Ok(()));
    assert_eq!(client.fetch_rating(&book, key), Ok(None));
}

fn keeps_an_annotation_over_text_until_it_is_deleted(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book();
    let id = uuid();

    assert_eq!(
        client.add_annotation(&book, &annotation(&earlier(), &later(), Some(&id)), key),
        Ok(id.clone())
    );
    assert_eq!(client.list_annotations(&book, key), Ok(vec![id.clone()]));

    let stored = client
        .get_annotation(&book, &id, key)
        .expect("The annotation should be held");
    assert_eq!(stored.start_location, earlier());
    assert_eq!(stored.end_location, later());

    assert_eq!(client.patch_annotation(&book, &id, "A note", key), Ok(()));
    assert_eq!(
        client
            .get_annotation(&book, &id, key)
            .ok()
            .and_then(|stored| stored.note),
        Some("A note".to_owned())
    );

    assert_eq!(client.delete_annotation(&book, &id, key), Ok(()));
    assert_eq!(
        client.get_annotation(&book, &id, key).err(),
        Some(ClientError::NotFound)
    );
    assert_eq!(
        client.delete_annotation(&book, &id, key),
        Err(ClientError::NotFound)
    );
}

fn refuses_an_annotation_that_is_not_a_stretch_of_text(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book();

    for (start, end) in [
        (earlier(), earlier()),
        (earlier(), at("0/0")),
        (later(), earlier()),
    ] {
        assert_eq!(
            client.add_annotation(&book, &annotation(&start, &end, None), key),
            Err(ClientError::BadRequest),
            "{start} .. {end} should be refused"
        );
    }
}

fn refuses_to_annotate_a_span_or_reuse_an_id_twice(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book();
    let id = uuid();

    client
        .add_annotation(&book, &annotation(&earlier(), &later(), Some(&id)), key)
        .expect("The first annotation should be taken");

    assert_eq!(
        client.add_annotation(&book, &annotation(&earlier(), &later(), Some(&uuid())), key),
        Err(ClientError::Conflict)
    );
    assert_eq!(
        client.add_annotation(&book, &annotation(&earlier(), &at("0/1/t0:3"), Some(&id)), key),
        Err(ClientError::Conflict)
    );
}

fn refuses_an_annotation_id_that_is_not_a_uuid(backend: &dyn Backend) {
    let book = backend.add_book();

    assert_eq!(
        backend.client().add_annotation(
            &book,
            &annotation(&earlier(), &later(), Some("not-a-uuid")),
            backend.key()
        ),
        Err(ClientError::BadRequest)
    );
}

fn stores_an_annotation_id_in_its_canonical_form(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book();
    let id = uuid();

    assert_eq!(
        client.add_annotation(
            &book,
            &annotation(&earlier(), &later(), Some(&id.to_uppercase())),
            key
        ),
        Ok(id.clone())
    );
    assert_eq!(client.list_annotations(&book, key), Ok(vec![id]));
}

fn holds_a_shelf_name_once_per_user(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let name = unique("Shelf ");
    let renamed = unique("Renamed ");

    let shelf = client
        .create_shelf(&name, None, None, key)
        .expect("The shelf should be created");
    assert_eq!(
        client.create_shelf(&name, None, None, key),
        Err(ClientError::Conflict)
    );
    assert_eq!(
        client.get_shelf_metadata(&shelf, key).map(|shelf| shelf.name),
        Ok(name)
    );

    assert_eq!(client.update_shelf_name(&shelf, &renamed, key), Ok(()));
    assert_eq!(
        client.get_shelf_metadata(&shelf, key).map(|shelf| shelf.name),
        Ok(renamed)
    );

    assert_eq!(client.delete_shelf(&shelf, key), Ok(()));
    assert_eq!(client.delete_shelf(&shelf, key), Err(ClientError::NotFound));
}

fn holds_each_book_on_a_shelf_once(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let book = backend.add_book();
    let shelf = client
        .create_shelf(&unique("Shelf "), None, None, key)
        .expect("The shelf should be created");

    assert_eq!(client.add_book_to_shelf(&shelf, &book, key), Ok(()));
    assert_eq!(
        client.add_book_to_shelf(&shelf, &book, key),
        Err(ClientError::Conflict)
    );
    assert_eq!(client.list_books_in_shelf(&shelf, key), Ok(vec![book.clone()]));

    assert_eq!(client.delete_book_from_shelf(&shelf, &book, key), Ok(()));
    assert_eq!(
        client.delete_book_from_shelf(&shelf, &book, key),
        Err(ClientError::NotFound)
    );
}

fn refuses_to_shelve_a_book_it_does_not_hold(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let shelf = client
        .create_shelf(&unique("Shelf "), None, None, key)
        .expect("The shelf should be created");

    assert_eq!(
        client.add_book_to_shelf(&shelf, MISSING, key),
        Err(ClientError::NotFound)
    );
}

fn finds_its_own_shelves_by_any_part_of_the_name(backend: &dyn Backend) {
    let client = backend.client();
    let key = backend.key();
    let word = unique("word");
    let shelf = client
        .create_shelf(&format!("My {word} shelf"), None, None, key)
        .expect("The shelf should be created");

    assert_eq!(
        client.search_shelves(backend.username(), &word.to_uppercase(), key),
        Ok(vec![shelf])
    );
    assert_eq!(
        client.search_shelves(&unique("nobody-"), &word, key),
        Err(ClientError::Forbidden)
    );
}

macro_rules! contract {
    ($($scenario:ident),* $(,)?) => {
        mod mock {
            $(
                #[test]
                fn $scenario() {
                    super::$scenario(&super::Mock::new());
                }
            )*
        }

        mod live {
            $(
                #[test]
                #[ignore = "needs a running Prosa at PROSA_URL"]
                fn $scenario() {
                    super::$scenario(&super::Live::new());
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
    keeps_a_rating_and_clears_it_at_zero,
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
