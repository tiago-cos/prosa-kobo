use crate::{
    app::state::service::unix_millis_to_string,
    client::{ProsaAnnotation, ProsaAnnotationRequest, ProsaLocation},
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Debug)]
pub struct CheckContentRequest {
    #[serde(rename = "ContentId")]
    pub content_id: String,
    pub etag: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAnnotationsResponse {
    pub annotations: Vec<Annotation>,
    pub next_page_offset_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchAnnotationsRequest {
    pub updated_annotations: Option<Vec<Annotation>>,
    pub deleted_annotation_ids: Option<Vec<String>>,
}

impl GetAnnotationsResponse {
    pub fn new(annotations: Vec<Annotation>) -> Self {
        Self {
            annotations,
            next_page_offset_token: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub client_last_modified_utc: String,
    pub id: String,
    pub location: AnnotationLocation,
    pub note_text: Option<String>,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationLocation {
    pub span: AnnotationSpan,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationSpan {
    pub chapter_filename: String,
    pub end_char: u32,
    pub end_path: String,
    pub start_char: u32,
    pub start_path: String,
}

impl From<ProsaAnnotation> for Annotation {
    fn from(annotation: ProsaAnnotation) -> Self {
        let start: ProsaLocation = annotation
            .start_location
            .parse()
            .expect("Failed to parse annotation start location");

        let end: ProsaLocation = annotation
            .end_location
            .parse()
            .expect("Failed to parse annotation end location");

        // The device's end character is exclusive, Prosa's is the last one covered.
        let span = AnnotationSpan {
            chapter_filename: start.source,
            end_char: end.offset.unwrap_or_default() + 1,
            end_path: end.path,
            start_char: start.offset.unwrap_or_default(),
            start_path: start.path,
        };

        let location = AnnotationLocation { span };

        let r#type = match annotation.note {
            Some(_) => "note".to_string(),
            None => "highlight".to_string(),
        };

        let now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get time since epoch")
            .as_millis()
            .try_into()
            .expect("Failed to get current timestamp");

        let now = unix_millis_to_string(now);

        Self {
            client_last_modified_utc: now,
            id: annotation.annotation_id,
            location,
            note_text: annotation.note,
            r#type,
        }
    }
}

impl From<Annotation> for ProsaAnnotationRequest {
    fn from(annotation: Annotation) -> Self {
        let span = annotation.location.span;

        let start = ProsaLocation::new(&span.chapter_filename, &span.start_path, Some(span.start_char));
        let end = ProsaLocation::new(
            &span.chapter_filename,
            &span.end_path,
            Some(span.end_char.saturating_sub(1)),
        );

        ProsaAnnotationRequest {
            start_location: start.to_string(),
            end_location: end.to_string(),
            note: annotation.note_text,
            annotation_id: Some(annotation.id),
        }
    }
}
