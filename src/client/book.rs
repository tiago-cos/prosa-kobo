use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

const MAX_BOOK_SIZE: u64 = 50 * 1024 * 1024;

pub struct BookClient {
    pub url: String,
    pub http: Client,
}

impl BookClient {
    pub async fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        let mut response = self
            .http
            .get(format!("{}/books/{book_id}", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?;

        let mut body: Vec<u8> = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            body.extend_from_slice(&chunk);

            if body.len() as u64 >= MAX_BOOK_SIZE {
                body.truncate(usize::try_from(MAX_BOOK_SIZE).unwrap_or(usize::MAX));
                break;
            }
        }

        Ok(body)
    }

    pub async fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), Error> {
        self.http
            .delete(format!("{}/books/{book_id}", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, Error> {
        self.http
            .get(format!("{}/books/{book_id}/file-metadata", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<ProsaBookFileMetadata>()
            .await
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaBookFileMetadata {
    pub owner_id: String,
    pub file_size: u64,
}
