use super::{
    data,
    models::{Annotation, CheckContentRequest, GetAnnotationsResponse, PatchAnnotationsRequest},
};
use crate::{app::ProsaClient, client::ProsaAnnotation};
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::RngCore;
use sqlx::SqlitePool;

pub async fn get_etag(pool: &SqlitePool, book_id: &str) -> String {
    data::get_etag(pool, book_id)
        .await
        .expect("Etag should be present")
}

pub async fn update_etag(pool: &SqlitePool, book_id: &str) -> () {
    let mut random = [0u8, 32];
    rand::rng().fill_bytes(&mut random);
    let etag = BASE64_STANDARD.encode(random);

    data::update_etag(pool, book_id, &etag).await;
}

//TODO check the device request when editing notes from a note
//TODO dont answer requests to books that the device should not have
pub async fn filter_changed(pool: &SqlitePool, books: Vec<CheckContentRequest>) -> Vec<String> {
    let mut changed: Vec<String> = Vec::new();

    for book in books {
        let etag = data::get_etag(pool, &book.content_id)
            .await
            .expect("Etag should be present");
        if etag != book.etag {
            changed.push(book.content_id);
        }
    }

    changed
}

pub async fn get_annotations(client: &ProsaClient, book_id: &str, api_key: &str) -> GetAnnotationsResponse {
    let annotation_ids = client
        .list_annotations(book_id, api_key)
        .expect("Annotation request should not fail");
    let mut annotations: Vec<ProsaAnnotation> = Vec::new();

    for id in annotation_ids {
        let annotation = client
            .get_annotation(book_id, &id, api_key)
            .expect("Annotation request should not fail");

        annotations.push(annotation);
    }

    let annotations: Vec<Annotation> = annotations.into_iter().map(|a| a.into()).collect();

    //TODO remove
    println!("{:#?}", annotations);

    GetAnnotationsResponse::new(annotations)
}

pub async fn patch_annotations(
    client: &ProsaClient,
    book_id: &str,
    request: PatchAnnotationsRequest,
    api_key: &str,
) {
    for annotation in request.updated_annotations.unwrap_or_default() {
        if let Err(_) = client.add_annotation(book_id, annotation.clone().into(), api_key) {
            client
                .patch_annotation(
                    book_id,
                    &annotation.id,
                    &annotation.note_text.unwrap_or("".to_string()),
                    api_key,
                )
                .expect("Annotation request should not fail");
        }
    }

    for annotation_id in request.deleted_annotation_ids.unwrap_or_default() {
        client
            .delete_annotation(book_id, &annotation_id, api_key)
            .expect("Annotation request should not fail");
    }
}

pub async fn delete_etag(pool: &SqlitePool, book_id: &str) {
    data::delete_etag(pool, book_id).await
}
