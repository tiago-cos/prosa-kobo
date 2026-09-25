use crate::app::error::unmapped;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumMessage, EnumProperty};

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum AnnotationError {
    #[strum(message = "InternalError")]
    #[strum(detailed_message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

impl From<sqlx::Error> for AnnotationError {
    fn from(error: sqlx::Error) -> Self {
        unmapped(&error, AnnotationError::InternalError)
    }
}

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
