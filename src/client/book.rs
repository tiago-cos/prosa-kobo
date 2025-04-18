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
}
