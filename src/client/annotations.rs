use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ureq::{Agent, Error};

pub struct AnnotationsClient {
    pub url: String,
    pub agent: Agent,
}

impl AnnotationsClient {
    pub fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, Error> {
        self.agent
            .get(format!("{}/books/{book_id}/annotations", self.url))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<Vec<String>>()
    }

    pub fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, Error> {
        self.agent
            .get(format!(
                "{}/books/{book_id}/annotations/{annotation_id}",
                self.url
            ))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaAnnotation>()
    }

    pub fn add_annotation(
        &self,
        book_id: &str,
        annotation: &ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, Error> {
        self.agent
            .post(format!("{}/books/{book_id}/annotations", self.url))
            .header("api-key", api_key)
            .send_json(annotation)?
            .body_mut()
            .read_to_string()
    }

    pub fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        let request = ProsaAnnotationPatch { note };

        self.agent
            .patch(format!(
                "{}/books/{book_id}/annotations/{annotation_id}",
                self.url
            ))
            .header("api-key", api_key)
            .send_json(request)?;

        Ok(())
    }

    pub fn delete_annotation(&self, book_id: &str, annotation_id: &str, api_key: &str) -> Result<(), Error> {
        self.agent
            .delete(format!(
                "{}/books/{book_id}/annotations/{annotation_id}",
                self.url
            ))
            .header("api-key", api_key)
            .call()?;

        Ok(())
    }
}

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
struct ProsaAnnotationPatch<'a> {
    note: &'a str,
}
