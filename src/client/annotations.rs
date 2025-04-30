use serde::{Deserialize, Serialize};
use serde_json::Value;
use ureq::{Agent, Error};

pub struct AnnotationsClient;

impl AnnotationsClient {
    pub fn list_annotations(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        api_key: &str,
    ) -> Result<Vec<String>, Error> {
        agent
            .get(format!("{}/books/{}/annotations", url, book_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<Vec<String>>()
    }

    pub fn get_annotation(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, Error> {
        agent
            .get(format!("{}/books/{}/annotations/{}", url, book_id, annotation_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaAnnotation>()
    }

    pub fn add_annotation(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        annotation: ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, Error> {
        agent
            .post(format!("{}/books/{}/annotations", url, book_id))
            .header("api-key", api_key)
            .send_json(annotation)?
            .body_mut()
            .read_to_string()
    }

    pub fn patch_annotation(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        let request = format!("{{\"note\": \"{}\"}}", note);
        agent
            .patch(format!("{}/books/{}/annotations/{}", url, book_id, annotation_id))
            .header("api-key", api_key)
            .send_json(serde_json::from_str::<Value>(&request).expect("Failed to serialize request"))?;

        Ok(())
    }

    pub fn delete_annotation(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        agent
            .delete(format!("{}/books/{}/annotations/{}", url, book_id, annotation_id))
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
