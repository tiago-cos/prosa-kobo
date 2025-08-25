use serde::Deserialize;
use std::io::Read;
use ureq::{Agent, Error};

pub struct BookClient {
    pub url: String,
    pub agent: Agent,
}

impl BookClient {
    pub fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        let mut body: Vec<u8> = Vec::new();
        self.agent
            .get(format!("{}/books/{book_id}", self.url))
            .header("api-key", api_key)
            .call()?
            .into_body()
            .into_reader()
            .take(50000000)
            .read_to_end(&mut body)?;

        Ok(body)
    }

    pub fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), Error> {
        self.agent
            .delete(format!("{}/books/{book_id}", self.url))
            .header("api-key", api_key)
            .call()?;

        Ok(())
    }

    pub fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, Error> {
        self.agent
            .get(format!("{}/books/{book_id}/file-metadata", self.url))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaBookFileMetadata>()
    }
}

#[derive(Deserialize, Debug)]
pub struct ProsaBookFileMetadata {
    pub owner_id: String,
    pub file_size: u64,
}
