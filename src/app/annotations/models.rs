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
    pub r#type: AnnotationType,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    Highlight,
    Note,
    #[serde(other)]
    Other,
}

impl AnnotationType {
    pub fn for_note(note: Option<&str>) -> Self {
        match note {
            Some(_) => AnnotationType::Note,
            None => AnnotationType::Highlight,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn names_a_highlight_and_a_note_as_the_device_does() {
        assert_eq!(
            serde_json::to_value(AnnotationType::Highlight).ok(),
            Some(json!("highlight"))
        );
        assert_eq!(
            serde_json::to_value(AnnotationType::Note).ok(),
            Some(json!("note"))
        );
    }

    #[test]
    fn accepts_a_type_it_does_not_know() {
        assert_eq!(
            serde_json::from_value::<AnnotationType>(json!("dogear")).ok(),
            Some(AnnotationType::Other)
        );
    }

    #[test]
    fn calls_an_annotation_with_a_note_a_note() {
        assert_eq!(AnnotationType::for_note(Some("A thought")), AnnotationType::Note);
        assert_eq!(AnnotationType::for_note(None), AnnotationType::Highlight);
    }
}
