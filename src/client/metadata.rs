use serde::Deserialize;
use ureq::{Agent, Error};

pub struct MetadataClient;

impl MetadataClient {
    pub fn fetch_metadata(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaMetadata, Error> {
        agent
            .get(format!("{}/books/{}/metadata", url, book_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaMetadata>()
    }

    pub fn fetch_size(&self, url: &str, agent: &Agent, book_id: &str, api_key: &str) -> Result<u64, Error> {
        agent
            .get(format!("{}/books/{}/size", url, book_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<u64>()
    }
}

#[derive(Deserialize, Debug)]
pub struct ProsaContributor {
    pub name: String,
    pub role: String,
}

#[derive(Deserialize, Debug)]
pub struct ProsaSeries {
    pub title: String,
    pub number: f32,
}

#[derive(Deserialize, Default, Debug)]
pub struct ProsaMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub publication_date: Option<i64>,
    pub isbn: Option<String>,
    pub contributors: Option<Vec<ProsaContributor>>,
    pub genres: Option<Vec<String>>,
    pub series: Option<ProsaSeries>,
    pub page_count: Option<i64>,
    pub language: Option<String>,
}
