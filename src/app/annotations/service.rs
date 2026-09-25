use super::{
    data,
    models::{
        Annotation, AnnotationLocation, AnnotationSpan, CheckContentRequest, GetAnnotationsResponse,
        PatchAnnotationsRequest,
    },
};
use crate::{
    app::{
        Kepubs, ProsaClient,
        error::KoboError,
        kepub::{self, KoboPosition},
        state::service::unix_millis_to_string,
    },
    client::{ProsaAnnotation, ProsaAnnotationRequest, prosa::ClientError},
    database::pool,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use log::warn;
use rand::RngCore;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_etag(book_id: &str) -> String {
    match data::get_etag(pool(), book_id).await {
        Some(tag) => return tag,
        None => update_etag(book_id).await,
    }

    data::get_etag(pool(), book_id)
        .await
        .expect("Etag should be present")
}

pub async fn update_etag(book_id: &str) {
    let mut random = [0u8; 32];
    rand::rng().fill_bytes(&mut random);
    let etag = BASE64_STANDARD.encode(random);

    data::update_etag(pool(), book_id, &etag).await;
}

pub async fn delete_etag(book_id: &str) {
    data::delete_etag(pool(), book_id).await;
}

pub async fn get_changed_annotations(books: Vec<CheckContentRequest>) -> Vec<String> {
    let mut changed: Vec<String> = Vec::new();

    for book in books {
        let Some(etag) = data::get_etag(pool(), &book.content_id).await else {
            data::update_etag(pool(), &book.content_id, &book.etag).await;
            continue;
        };

        if etag != book.etag {
            changed.push(book.content_id);
        }
    }

    changed
}

pub async fn get_annotations(
    kepubs: &Kepubs,
    client: &ProsaClient,
    book_id: &str,
    api_key: &str,
) -> Result<GetAnnotationsResponse, KoboError> {
    let annotation_ids = client.list_annotations(book_id, api_key)?;
    let mut stored: Vec<ProsaAnnotation> = Vec::new();

    for id in annotation_ids {
        stored.push(client.get_annotation(book_id, &id, api_key)?);
    }

    if stored.is_empty() {
        return Ok(GetAnnotationsResponse::new(Vec::new()));
    }

    let kepub = kepub::get_kepub(kepubs, client, book_id, api_key).await?;

    let annotations = stored
        .into_iter()
        .filter_map(|annotation| to_kobo_annotation(&kepub, annotation))
        .collect();

    Ok(GetAnnotationsResponse::new(annotations))
}

pub async fn patch_annotations(
    kepubs: &Kepubs,
    client: &ProsaClient,
    book_id: &str,
    request: PatchAnnotationsRequest,
    api_key: &str,
) -> Result<(), KoboError> {
    let updated = request.updated_annotations.unwrap_or_default();

    if !updated.is_empty() {
        let kepub = kepub::get_kepub(kepubs, client, book_id, api_key).await?;

        for annotation in updated {
            let Some(request) = to_prosa_annotation(&kepub, &annotation) else {
                continue;
            };

            let result = client.add_annotation(book_id, &request, api_key);
            let note = &annotation.note_text.unwrap_or_default();

            if let Err(ClientError::Conflict) = result {
                client.patch_annotation(book_id, &annotation.id, note, api_key)?;
            } else {
                result?;
            }
        }
    }

    for annotation_id in request.deleted_annotation_ids.unwrap_or_default() {
        client.delete_annotation(book_id, &annotation_id, api_key)?;
    }

    Ok(())
}

fn to_kobo_annotation(kepub: &[u8], annotation: ProsaAnnotation) -> Option<Annotation> {
    let start = kepub::to_kobo_position(kepub, &annotation.start_location)?;
    let end = kepub::to_kobo_position(kepub, &annotation.end_location)?;

    let span = AnnotationSpan {
        chapter_filename: start.chapter.clone(),
        end_char: end.offset,
        end_path: end.selector(),
        start_char: start.offset,
        start_path: start.selector(),
    };

    let now: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_millis()
        .try_into()
        .expect("Failed to get current timestamp");

    Some(Annotation {
        client_last_modified_utc: unix_millis_to_string(now),
        id: annotation.annotation_id,
        location: AnnotationLocation { span },
        r#type: match annotation.note {
            Some(_) => "note".to_owned(),
            None => "highlight".to_owned(),
        },
        note_text: annotation.note,
    })
}

fn to_prosa_annotation(kepub: &[u8], annotation: &Annotation) -> Option<ProsaAnnotationRequest> {
    let span = &annotation.location.span;

    let start = KoboPosition::new(&span.chapter_filename, &span.start_path, span.start_char);
    let end = KoboPosition::new(&span.chapter_filename, &span.end_path, span.end_char);

    let (Some(start_location), Some(end_location)) = (
        kepub::to_prosa_location(kepub, &start),
        kepub::to_prosa_location(kepub, &end),
    ) else {
        warn!("Dropping annotation {}: its span does not resolve", annotation.id);
        return None;
    };

    Some(ProsaAnnotationRequest {
        start_location,
        end_location,
        note: annotation.note_text.clone(),
        annotation_id: Some(annotation.id.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::kepub::KepubCache,
        client::mock::{MockProsaClient, ProsaMethod},
    };
    use regex::Regex;
    use std::{fs, io::Cursor, sync::Arc};

    const BOOK: &str = "book";
    const ANNOTATION: &str = "0b7f8a4e-5c1d-4e2a-9f3b-6d8c1a2e4f50";
    const EPUB: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/The_Great_Gatsby.epub"
    );

    struct Fixture {
        cache: Kepubs,
        client: Arc<MockProsaClient>,
        chapter: String,
        start: String,
        end: String,
    }

    impl Fixture {
        /// Reads a chapter and two of its koboSpans out of the converted book,
        /// so the annotations under test address positions that really exist.
        async fn new() -> Self {
            let client = Arc::new(MockProsaClient::new());
            client.seed_file(BOOK, fs::read(EPUB).expect("Failed to read the test epub"));

            let cache: Kepubs = Arc::new(KepubCache::new(64 * 1024 * 1024));
            let prosa = Arc::clone(&client) as ProsaClient;

            let kepub = kepub::get_kepub(&cache, &prosa, BOOK, "key")
                .await
                .expect("Failed to convert the test epub");

            let (chapter, start, end) = spanned_chapter(&kepub);

            Self {
                cache,
                client,
                chapter,
                start,
                end,
            }
        }

        fn prosa(&self) -> ProsaClient {
            Arc::clone(&self.client) as ProsaClient
        }

        fn annotation(&self, note: Option<&str>) -> Annotation {
            Annotation {
                client_last_modified_utc: "2026-01-01T00:00:00.0000000Z".to_owned(),
                id: ANNOTATION.to_owned(),
                location: AnnotationLocation {
                    span: AnnotationSpan {
                        chapter_filename: self.chapter.clone(),
                        end_char: 1,
                        end_path: selector(&self.end),
                        start_char: 0,
                        start_path: selector(&self.start),
                    },
                },
                note_text: note.map(str::to_owned),
                r#type: "note".to_owned(),
            }
        }

        async fn patch(&self, request: PatchAnnotationsRequest) {
            patch_annotations(&self.cache, &self.prosa(), BOOK, request, "key")
                .await
                .expect("Failed to patch annotations");
        }
    }

    fn spanned_chapter(kepub: &[u8]) -> (String, String, String) {
        let mut archive = zip::ZipArchive::new(Cursor::new(kepub)).expect("Failed to open the kepub");
        let id = Regex::new(r#"id="(kobo\.\d+\.\d+)""#).expect("Failed to build regex");

        let documents: Vec<String> = archive
            .file_names()
            .filter(|name| name.contains(".xhtml") || name.contains(".htm"))
            .map(str::to_owned)
            .collect();

        for name in documents {
            let mut body = String::new();
            let mut file = archive.by_name(&name).expect("Failed to open a content document");

            if std::io::Read::read_to_string(&mut file, &mut body).is_err() {
                continue;
            }

            let mut spans = id.captures_iter(&body).map(|span| span[1].to_owned());

            if let (Some(start), Some(end)) = (spans.next(), spans.next()) {
                return (name, start, end);
            }
        }

        panic!("The converted book has no spanned chapter to build fixtures from");
    }

    fn selector(span: &str) -> String {
        format!("span#{}", span.replace('.', r"\."))
    }

    fn update(annotations: Vec<Annotation>) -> PatchAnnotationsRequest {
        PatchAnnotationsRequest {
            updated_annotations: Some(annotations),
            deleted_annotation_ids: None,
        }
    }

    #[tokio::test]
    async fn creates_an_annotation_under_the_id_the_device_chose() {
        let fixture = Fixture::new().await;
        fixture
            .patch(update(vec![fixture.annotation(Some("A note"))]))
            .await;

        let stored = fixture.client.stored_annotations(BOOK);
        let annotation = stored.first().expect("Expected one annotation");

        assert_eq!(annotation.annotation_id, ANNOTATION);
        assert_eq!(annotation.note.as_deref(), Some("A note"));
        assert_eq!(fixture.client.call_count(ProsaMethod::PatchAnnotation), 0);
    }

    #[tokio::test]
    async fn a_one_character_highlight_keeps_its_two_ends_apart() {
        let fixture = Fixture::new().await;
        let mut annotation = fixture.annotation(None);
        annotation.location.span.end_path = annotation.location.span.start_path.clone();
        annotation.location.span.start_char = 4;
        annotation.location.span.end_char = 5;

        fixture.patch(update(vec![annotation])).await;

        let stored = fixture.client.stored_annotations(BOOK);
        let annotation = stored.first().expect("Expected one annotation");

        assert_ne!(annotation.start_location, annotation.end_location);
        assert!(annotation.start_location.ends_with(":4"));
        assert!(annotation.end_location.ends_with(":5"));
    }

    #[tokio::test]
    async fn stores_a_prosa_location_rather_than_the_span_the_device_sent() {
        let fixture = Fixture::new().await;
        fixture.patch(update(vec![fixture.annotation(None)])).await;

        let stored = fixture.client.stored_annotations(BOOK);
        let annotation = stored.first().expect("Expected one annotation");

        assert!(
            annotation
                .start_location
                .starts_with(&format!("{}#", fixture.chapter))
        );
        assert!(!annotation.start_location.contains("kobo."));
        assert!(!annotation.end_location.contains("kobo."));
    }

    #[tokio::test]
    async fn an_annotation_round_trips_back_to_the_span_it_came_from() {
        let fixture = Fixture::new().await;
        let sent = fixture.annotation(None);

        fixture.patch(update(vec![sent.clone()])).await;

        let returned = get_annotations(&fixture.cache, &fixture.prosa(), BOOK, "key")
            .await
            .expect("Failed to get annotations");

        let returned = returned.annotations.first().expect("Expected one annotation");

        assert_eq!(
            returned.location.span.chapter_filename,
            sent.location.span.chapter_filename
        );
        assert_eq!(returned.location.span.start_path, sent.location.span.start_path);
        assert!(returned.location.span.start_path.starts_with("span#kobo"));
        assert_eq!(returned.location.span.start_char, sent.location.span.start_char);
        assert_eq!(returned.location.span.end_path, sent.location.span.end_path);
        assert_eq!(returned.location.span.end_char, sent.location.span.end_char);
    }

    #[tokio::test]
    async fn translates_an_annotation_prosa_already_held() {
        let fixture = Fixture::new().await;
        let kepub = kepub::get_kepub(&fixture.cache, &fixture.prosa(), BOOK, "key")
            .await
            .expect("Failed to convert the test epub");

        let caret = |offset| {
            let position = KoboPosition::new(&fixture.chapter, &fixture.start, offset);
            kepub::to_prosa_location(&kepub, &position).expect("Expected a location")
        };

        fixture.client.seed_annotation(
            BOOK,
            ProsaAnnotation {
                annotation_id: "stored".to_owned(),
                start_location: caret(0),
                end_location: caret(1),
                note: None,
            },
        );

        let returned = get_annotations(&fixture.cache, &fixture.prosa(), BOOK, "key")
            .await
            .expect("Failed to get annotations");

        let returned = returned.annotations.first().expect("Expected one annotation");

        assert_eq!(returned.id, "stored");
        assert_eq!(returned.r#type, "highlight");
        assert_eq!(returned.location.span.chapter_filename, fixture.chapter);
        assert_eq!(returned.location.span.start_path, selector(&fixture.start));
        assert_eq!(returned.location.span.start_char, 0);
        assert_eq!(returned.location.span.end_char, 1);
    }

    #[tokio::test]
    async fn updates_the_note_when_the_annotation_already_exists() {
        let fixture = Fixture::new().await;

        fixture.patch(update(vec![fixture.annotation(None)])).await;
        fixture
            .patch(update(vec![fixture.annotation(Some("A later note"))]))
            .await;

        let stored = fixture.client.stored_annotations(BOOK);
        let annotation = stored.first().expect("Expected one annotation");

        assert_eq!(stored.len(), 1);
        assert_eq!(annotation.note.as_deref(), Some("A later note"));
        assert_eq!(fixture.client.call_count(ProsaMethod::PatchAnnotation), 1);
    }

    #[tokio::test]
    async fn drops_an_annotation_whose_span_the_book_does_not_have() {
        let fixture = Fixture::new().await;
        let mut annotation = fixture.annotation(None);
        annotation.location.span.start_path = selector("kobo.99999.1");

        fixture.patch(update(vec![annotation])).await;

        assert!(fixture.client.stored_annotations(BOOK).is_empty());
        assert_eq!(fixture.client.call_count(ProsaMethod::AddAnnotation), 0);
    }

    #[tokio::test]
    async fn deletes_the_annotations_the_device_removed() {
        let fixture = Fixture::new().await;
        fixture.patch(update(vec![fixture.annotation(None)])).await;

        fixture
            .patch(PatchAnnotationsRequest {
                updated_annotations: None,
                deleted_annotation_ids: Some(vec![ANNOTATION.to_owned()]),
            })
            .await;

        assert!(fixture.client.stored_annotations(BOOK).is_empty());
    }
}
