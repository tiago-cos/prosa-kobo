use serde::Deserialize;
use std::io::Read;
use ureq::{Agent, Error};

pub struct BookClient;

impl BookClient {
    pub fn download_book(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        api_key: &str,
    ) -> Result<Vec<u8>, Error> {
        let mut body: Vec<u8> = Vec::new();
        agent
            .get(format!("{}/books/{}", url, book_id))
            .header("api-key", api_key)
            .call()?
            .into_body()
            .into_reader()
            .take(50000000)
            .read_to_end(&mut body)?;

        Ok(body)
    }

    pub fn delete_book(&self, url: &str, agent: &Agent, book_id: &str, api_key: &str) -> Result<(), Error> {
        agent
            .delete(format!("{}/books/{}", url, book_id))
            .header("api-key", api_key)
            .call()?;

        Ok(())
    }

    pub fn fetch_book_file_metadata(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, Error> {
        agent
            .get(format!("{}/books/{}/file-metadata", url, book_id))
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
