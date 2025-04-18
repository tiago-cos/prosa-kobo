use std::io::Read;
use ureq::Error;

pub struct CoverClient;

impl CoverClient {
    pub fn download_cover(&self, url: &str, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        let mut body: Vec<u8> = Vec::new();
        ureq::get(format!("{}/books/{}/cover", url, book_id))
            .header("api-key", api_key)
            .call()?
            .into_body()
            .into_reader()
            .take(50000000)
            .read_to_end(&mut body)?;

        Ok(body)
    }
}
