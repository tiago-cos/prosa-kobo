use super::{
    data,
    models::{Annotation, CheckContentRequest, GetAnnotationsResponse, PatchAnnotationsRequest},
};
use crate::{
    app::error::KoboError,
    client::{
        ProsaAnnotation, ProsaAnnotationRequest,
        prosa::{ClientError, ProsaApi},
    },
};
use base64::{Engine, prelude::BASE64_STANDARD};
use rand::RngCore;
use sqlx::SqlitePool;

pub async fn get_etag(pool: &SqlitePool, book_id: &str) -> String {
    match data::get_etag(pool, book_id).await {
        Some(tag) => return tag,
        None => update_etag(pool, book_id).await,
    }

    data::get_etag(pool, book_id)
        .await
        .expect("Etag should be present")
}

pub async fn update_etag(pool: &SqlitePool, book_id: &str) -> () {
    let mut random = [0u8; 32];
    rand::rng().fill_bytes(&mut random);
    let etag = BASE64_STANDARD.encode(random);

    data::update_etag(pool, book_id, &etag).await;
}

pub async fn delete_etag(pool: &SqlitePool, book_id: &str) {
    data::delete_etag(pool, book_id).await;
}

pub async fn get_changed_annotations(pool: &SqlitePool, books: Vec<CheckContentRequest>) -> Vec<String> {
    let mut changed: Vec<String> = Vec::new();

    for book in books {
        let Some(etag) = data::get_etag(pool, &book.content_id).await else {
            data::update_etag(pool, &book.content_id, &book.etag).await;
            continue;
        };

        if etag != book.etag {
            changed.push(book.content_id);
        }
    }

    changed
}

pub fn get_annotations(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<GetAnnotationsResponse, ClientError> {
    let annotation_ids = client.list_annotations(book_id, api_key)?;
    let mut annotations: Vec<ProsaAnnotation> = Vec::new();

    for id in annotation_ids {
        let annotation = client.get_annotation(book_id, &id, api_key)?;
        annotations.push(annotation);
    }

    let annotations: Vec<Annotation> = annotations.into_iter().map(Into::into).collect();

    Ok(GetAnnotationsResponse::new(annotations))
}

pub fn patch_annotations(
    client: &dyn ProsaApi,
    book_id: &str,
    request: PatchAnnotationsRequest,
    api_key: &str,
) -> Result<(), KoboError> {
    for annotation in request.updated_annotations.unwrap_or_default() {
        let request: ProsaAnnotationRequest = annotation.clone().into();
        let result = client.add_annotation(book_id, &request, api_key);
        let note = &annotation.note_text.unwrap_or_default();

        if let Err(ClientError::Conflict) = result {
            client.patch_annotation(book_id, &annotation.id, note, api_key)?;
        } else {
            result?;
        }
    }

    for annotation_id in request.deleted_annotation_ids.unwrap_or_default() {
        client.delete_annotation(book_id, &annotation_id, api_key)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::annotations::models::{Annotation, AnnotationLocation, AnnotationSpan},
        client::mock::{MockProsaClient, ProsaMethod},
    };

    fn prosa_annotation(note: Option<&str>) -> ProsaAnnotation {
        ProsaAnnotation {
            annotation_id: "annotation".to_owned(),
            start_location: "OEBPS/chapter-001.xhtml#0/2/t1:44".to_owned(),
            end_location: "OEBPS/chapter-001.xhtml#0/3/t0:12".to_owned(),
            note: note.map(str::to_owned),
        }
    }

    fn kobo_annotation(note: Option<&str>) -> Annotation {
        Annotation {
            client_last_modified_utc: "2026-01-01T00:00:00.0000000Z".to_owned(),
            id: "annotation".to_owned(),
            location: AnnotationLocation {
                span: AnnotationSpan {
                    chapter_filename: "OEBPS/chapter-001.xhtml".to_owned(),
                    end_char: 13,
                    end_path: "0/3/t0".to_owned(),
                    start_char: 44,
                    start_path: "0/2/t1".to_owned(),
                },
            },
            note_text: note.map(str::to_owned),
            r#type: "note".to_owned(),
        }
    }

    #[test]
    fn creates_an_annotation_under_the_id_the_device_chose() {
        let client = MockProsaClient::new();
        client.seed_book("book");

        let request = PatchAnnotationsRequest {
            updated_annotations: Some(vec![kobo_annotation(Some("A note"))]),
            deleted_annotation_ids: None,
        };

        patch_annotations(&client, "book", request, "key").expect("Failed to patch annotations");

        let stored = client.stored_annotations("book");
        let annotation = stored.first().expect("Expected one annotation");

        assert_eq!(annotation.annotation_id, "annotation");
        assert_eq!(annotation.note.as_deref(), Some("A note"));
        assert_eq!(client.call_count(ProsaMethod::PatchAnnotation), 0);
    }

    #[test]
    fn updates_the_note_when_the_annotation_already_exists() {
        let client = MockProsaClient::new();
        client.seed_annotation("book", prosa_annotation(None));

        let request = PatchAnnotationsRequest {
            updated_annotations: Some(vec![kobo_annotation(Some("A later note"))]),
            deleted_annotation_ids: None,
        };

        patch_annotations(&client, "book", request, "key").expect("Failed to patch annotations");

        let stored = client.stored_annotations("book");
        let annotation = stored.first().expect("Expected one annotation");

        assert_eq!(stored.len(), 1);
        assert_eq!(annotation.note.as_deref(), Some("A later note"));
        assert_eq!(client.call_count(ProsaMethod::PatchAnnotation), 1);
    }

    #[test]
    fn deletes_the_annotations_the_device_removed() {
        let client = MockProsaClient::new();
        client.seed_annotation("book", prosa_annotation(None));

        let request = PatchAnnotationsRequest {
            updated_annotations: None,
            deleted_annotation_ids: Some(vec!["annotation".to_owned()]),
        };

        patch_annotations(&client, "book", request, "key").expect("Failed to patch annotations");

        assert!(client.stored_annotations("book").is_empty());
    }
}
