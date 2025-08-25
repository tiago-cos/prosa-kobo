use std::io::Read;
use ureq::{Agent, Error};

pub struct CoverClient {
    pub url: String,
    pub agent: Agent,
}

impl CoverClient {
    pub fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        let mut body: Vec<u8> = Vec::new();
        self.agent
            .get(format!("{}/books/{book_id}/cover", self.url))
            .header("api-key", api_key)
            .call()?
            .into_body()
            .into_reader()
            .take(50000000)
            .read_to_end(&mut body)?;

        Ok(body)
    }
}
