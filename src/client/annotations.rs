use serde::{Deserialize, Serialize};
use serde_json::Value;
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
        annotation: ProsaAnnotationRequest,
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
        let request = format!("{{\"note\": \"{note}\"}}");
        self.agent
            .patch(format!(
                "{}/books/{book_id}/annotations/{annotation_id}",
                self.url
            ))
            .header("api-key", api_key)
            .send_json(serde_json::from_str::<Value>(&request).expect("Failed to serialize request"))?;

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

#[derive(Deserialize, Debug)]
pub struct ProsaAnnotation {
    pub annotation_id: String,
    pub source: String,
    pub start_tag: String,
    pub end_tag: String,
    pub start_char: u32,
    pub end_char: u32,
    pub note: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ProsaAnnotationRequest {
    pub source: String,
    pub start_tag: String,
    pub end_tag: String,
    pub start_char: u32,
    pub end_char: u32,
    pub note: Option<String>,
}
