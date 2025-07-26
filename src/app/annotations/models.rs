use crate::{
    app::state::service::unix_millis_to_string,
    client::{ProsaAnnotation, ProsaAnnotationRequest},
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
        let span = AnnotationSpan {
            chapter_filename: annotation.source,
            end_char: annotation.end_char + 1,
            end_path: format!("span#{}", annotation.end_tag)
                .to_string()
                .replace(".", "\\."),
            start_char: annotation.start_char,
            start_path: format!("span#{}", annotation.start_tag)
                .to_string()
                .replace(".", "\\."),
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
        ProsaAnnotationRequest {
            source: annotation.location.span.chapter_filename,
            start_tag: annotation
                .location
                .span
                .start_path
                .strip_prefix("span#")
                .expect("Failed to parse annotation source")
                .to_string()
                .replace("\\.", "."),
            end_tag: annotation
                .location
                .span
                .end_path
                .strip_prefix("span#")
                .expect("Failed to parse annotation source")
                .to_string()
                .replace("\\.", "."),
            start_char: annotation.location.span.start_char,
            end_char: annotation.location.span.end_char - 1,
            note: annotation.note_text,
        }
    }
}
