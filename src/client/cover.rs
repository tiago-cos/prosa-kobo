use reqwest::{Client, Error};

const MAX_COVER_SIZE: u64 = 10 * 1024 * 1024;

pub struct CoverClient {
    pub url: String,
    pub http: Client,
}

impl CoverClient {
    pub async fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        let mut response = self
            .http
            .get(format!("{}/books/{book_id}/cover", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?;

        let mut body: Vec<u8> = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            body.extend_from_slice(&chunk);

            if body.len() as u64 >= MAX_COVER_SIZE {
                body.truncate(usize::try_from(MAX_COVER_SIZE).unwrap_or(usize::MAX));
                break;
            }
        }

        Ok(body)
    }
}
