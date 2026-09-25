mod common;

use axum::http::{Method, StatusCode};
use common::{Device, Harness, USER, body_bytes, body_json};
use prosa_kobo::client::{mock::ProsaMethod, prosa::ClientError};
use serde_json::{Value, json};

fn items(book_ids: &[&str]) -> Value {
    json!(
        book_ids
            .iter()
            .map(|id| json!({ "RevisionId": id, "Type": "ProductRevisionTagItem" }))
            .collect::<Vec<_>>()
    )
}

async fn with_shelf(books: &[&str]) -> (Harness, Device) {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    for book in books {
        harness.client.seed_book(book);
    }
    harness.client.seed_shelf("shelf", "To read", books);

    (harness, device)
}

fn created(harness: &Harness, name: &str) -> bool {
    harness
        .client
        .calls_to(ProsaMethod::CreateShelf)
        .iter()
        .any(|call| call.arguments.first().is_some_and(|argument| argument == name))
}

#[tokio::test]
async fn creates_the_shelf_the_device_names() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags"),
            json!({ "Name": "Favourites", "Items": [] }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert!(created(&harness, "Favourites"));
}

#[tokio::test]
async fn answers_with_the_id_prosa_gave_the_shelf_as_plain_text() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags"),
            json!({ "Name": "Favourites", "Items": [] }),
        )
        .await;
    let id = String::from_utf8(body_bytes(response).await).expect("The id should be text");

    assert_eq!(
        harness.client.stored_shelf_name(&id).as_deref(),
        Some("Favourites")
    );
}

#[tokio::test]
async fn places_the_books_a_new_shelf_is_created_with() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness.client.seed_book("first").seed_book("second");

    harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags"),
            json!({ "Name": "Favourites", "Items": items(&["first", "second"]) }),
        )
        .await;

    let added: Vec<String> = harness
        .client
        .calls_to(ProsaMethod::AddBookToShelf)
        .into_iter()
        .map(|call| call.arguments[1].clone())
        .collect();

    assert_eq!(added, vec!["first".to_owned(), "second".to_owned()]);
}

async fn create(harness: &Harness, device: &Device, name: &str, books: &[&str]) -> String {
    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags"),
            json!({ "Name": name, "Items": items(books) }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    String::from_utf8(body_bytes(response).await).expect("The id should be text")
}

#[tokio::test]
async fn reuses_the_shelf_already_holding_the_name() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness
        .client
        .seed_book("new")
        .seed_owned_shelf(USER, "existing", "Favourites", &["old"]);

    let id = create(&harness, &device, "Favourites", &["new"]).await;

    assert_eq!(id, "existing");
    assert_eq!(
        harness.client.stored_shelf_books("existing"),
        vec!["old".to_owned(), "new".to_owned()]
    );
}

#[tokio::test]
async fn reuses_only_a_shelf_with_exactly_that_name() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness
        .client
        .seed_owned_shelf(USER, "a-longer", "Favourites of 2026", &[])
        .seed_owned_shelf(USER, "b-lowercase", "favourites", &[])
        .seed_owned_shelf(USER, "c-exact", "Favourites", &[]);

    assert_eq!(create(&harness, &device, "Favourites", &[]).await, "c-exact");
}

#[tokio::test]
async fn keeps_another_users_shelf_of_the_same_name_out_of_it() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness
        .client
        .seed_book("mine")
        .seed_owned_shelf("someone-else", "theirs", "Favourites", &["theirs"]);

    let id = create(&harness, &device, "Favourites", &["mine"]).await;

    assert_ne!(id, "theirs");
    assert_eq!(
        harness.client.stored_shelf_books("theirs"),
        vec!["theirs".to_owned()]
    );
    assert_eq!(harness.client.stored_shelf_books(&id), vec!["mine".to_owned()]);
}

#[tokio::test]
async fn passes_on_a_clash_it_cannot_place() {
    let harness = Harness::new().await;
    let device = harness.linked().await;
    harness
        .client
        .fail(ProsaMethod::CreateShelf, ClientError::Conflict);

    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags"),
            json!({ "Name": "Favourites", "Items": [] }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn renames_the_shelf() {
    let (harness, device) = with_shelf(&[]).await;

    let response = harness
        .json(
            Method::PUT,
            &device.at("/v1/library/tags/shelf"),
            json!({ "Name": "Finished" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        harness.client.stored_shelf_name("shelf").as_deref(),
        Some("Finished")
    );
}

#[tokio::test]
async fn answers_not_found_when_renaming_a_shelf_prosa_does_not_hold() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness
        .json(
            Method::PUT,
            &device.at("/v1/library/tags/shelf"),
            json!({ "Name": "Finished" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn deletes_the_shelf() {
    let (harness, device) = with_shelf(&["book"]).await;

    let response = harness
        .request(Method::DELETE, &device.at("/v1/library/tags/shelf"), &[])
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(harness.client.stored_shelf_name("shelf").is_none());
}

#[tokio::test]
async fn deleting_a_shelf_prosa_no_longer_holds_still_succeeds() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness
        .request(Method::DELETE, &device.at("/v1/library/tags/shelf"), &[])
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn adds_the_books_and_names_them_back() {
    let (harness, device) = with_shelf(&[]).await;
    harness.client.seed_book("first").seed_book("second");

    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags/shelf/items"),
            json!({ "Items": items(&["first", "second"]) }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(body_json(response).await, json!(["first", "second"]));
    assert_eq!(
        harness.client.stored_shelf_books("shelf"),
        vec!["first".to_owned(), "second".to_owned()]
    );
}

#[tokio::test]
async fn adding_a_book_the_shelf_already_holds_neither_fails_nor_repeats_it() {
    let (harness, device) = with_shelf(&["book"]).await;

    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags/shelf/items"),
            json!({ "Items": items(&["book"]) }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        harness.client.stored_shelf_books("shelf"),
        vec!["book".to_owned()]
    );
}

#[tokio::test]
async fn answers_not_found_when_adding_to_a_shelf_prosa_does_not_hold() {
    let harness = Harness::new().await;
    let device = harness.linked().await;

    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags/shelf/items"),
            json!({ "Items": items(&["book"]) }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn removes_the_books() {
    let (harness, device) = with_shelf(&["first", "second"]).await;

    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags/shelf/items/delete"),
            json!({ "Items": items(&["first"]) }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        harness.client.stored_shelf_books("shelf"),
        vec!["second".to_owned()]
    );
}

#[tokio::test]
async fn removing_a_book_the_shelf_does_not_hold_still_succeeds() {
    let (harness, device) = with_shelf(&["first"]).await;

    let response = harness
        .json(
            Method::POST,
            &device.at("/v1/library/tags/shelf/items/delete"),
            json!({ "Items": items(&["second"]) }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        harness.client.stored_shelf_books("shelf"),
        vec!["first".to_owned()]
    );
}

#[tokio::test]
async fn passes_on_prosa_refusing_the_key() {
    let (harness, device) = with_shelf(&[]).await;
    harness
        .client
        .fail(ProsaMethod::UpdateShelfName, ClientError::Forbidden);

    let response = harness
        .json(
            Method::PUT,
            &device.at("/v1/library/tags/shelf"),
            json!({ "Name": "Finished" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
