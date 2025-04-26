use super::{
    data,
    models::{
        Annotation, AnnotationError, CheckContentRequest, GetAnnotationsResponse, PatchAnnotationsRequest,
    },
};
use crate::{
    app::{error::KoboError, ProsaClient},
    client::{prosa::ClientError, ProsaAnnotation},
};
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::RngCore;
use sqlx::SqlitePool;

pub async fn get_etag(pool: &SqlitePool, book_id: &str) -> Result<String, AnnotationError> {
    data::get_etag(pool, book_id)
        .await
        .ok_or_else(|| AnnotationError::AnnotationNotFound)
}

pub async fn update_etag(pool: &SqlitePool, book_id: &str) -> () {
    let mut random = [0u8, 32];
    rand::rng().fill_bytes(&mut random);
    let etag = BASE64_STANDARD.encode(random);

    data::update_etag(pool, book_id, &etag).await;
}

pub async fn delete_etag(pool: &SqlitePool, book_id: &str) {
    data::delete_etag(pool, book_id).await
}

pub async fn get_changed_annotations(pool: &SqlitePool, books: Vec<CheckContentRequest>) -> Vec<String> {
    let mut changed: Vec<String> = Vec::new();

    for book in books {
        let etag = match data::get_etag(pool, &book.content_id).await {
            Some(tag) => tag,
            None => continue,
        };

        if etag != book.etag {
            changed.push(book.content_id);
        }
    }

    changed
}

pub async fn get_annotations(
    client: &ProsaClient,
    book_id: &str,
    api_key: &str,
) -> Result<GetAnnotationsResponse, ClientError> {
    let annotation_ids = client.list_annotations(book_id, api_key)?;
    let mut annotations: Vec<ProsaAnnotation> = Vec::new();

    for id in annotation_ids {
        let annotation = client.get_annotation(book_id, &id, api_key)?;
        annotations.push(annotation);
    }

    let annotations: Vec<Annotation> = annotations.into_iter().map(|a| a.into()).collect();

    //TODO remove
    println!("{:#?}", annotations);

    Ok(GetAnnotationsResponse::new(annotations))
}

pub async fn patch_annotations(
    client: &ProsaClient,
    book_id: &str,
    request: PatchAnnotationsRequest,
    api_key: &str,
) -> Result<(), KoboError> {
    for annotation in request.updated_annotations.unwrap_or_default() {
        let result = client.add_annotation(book_id, annotation.clone().into(), api_key);
        let note = &annotation.note_text.unwrap_or_default();

        if let Err(ClientError::Conflict) = result {
            client.patch_annotation(book_id, &annotation.id, note, api_key)?;
        }
    }

    for annotation_id in request.deleted_annotation_ids.unwrap_or_default() {
        client.delete_annotation(book_id, &annotation_id, api_key)?;
    }

    Ok(())
}
