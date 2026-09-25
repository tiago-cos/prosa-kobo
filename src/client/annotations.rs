use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub struct AnnotationsClient {
    pub url: String,
    pub http: Client,
}

impl AnnotationsClient {
    pub async fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, Error> {
        self.http
            .get(format!("{}/books/{book_id}/annotations", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<String>>()
            .await
    }

    pub async fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, Error> {
        self.http
            .get(format!(
                "{}/books/{book_id}/annotations/{annotation_id}",
                self.url
            ))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<ProsaAnnotation>()
            .await
    }

    pub async fn add_annotation(
        &self,
        book_id: &str,
        annotation: &ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, Error> {
        self.http
            .post(format!("{}/books/{book_id}/annotations", self.url))
            .header("api-key", api_key)
            .json(&annotation)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        let request = ProsaAnnotationPatch { note };

        self.http
            .patch(format!(
                "{}/books/{book_id}/annotations/{annotation_id}",
                self.url
            ))
            .header("api-key", api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn delete_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        self.http
            .delete(format!(
                "{}/books/{book_id}/annotations/{annotation_id}",
                self.url
            ))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?;

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
