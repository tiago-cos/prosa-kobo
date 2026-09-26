use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaAnnotation {
    pub annotation_id: String,
    pub start_location: String,
    pub end_location: String,
    pub note: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaAnnotationRequest {
    pub start_location: String,
    pub end_location: String,
    pub note: Option<String>,
    /// Prosa generates one when this is absent, but the Kobo device names its
    /// own annotations, so the middleware always supplies the device's ID.
    pub annotation_id: Option<String>,
}

#[derive(Serialize, Debug)]
pub(super) struct ProsaAnnotationPatch<'a> {
    pub(super) note: &'a str,
}
