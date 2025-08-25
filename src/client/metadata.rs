use serde::Deserialize;
use ureq::{Agent, Error};

pub struct MetadataClient {
    pub url: String,
    pub agent: Agent,
}

impl MetadataClient {
    pub fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, Error> {
        self.agent
            .get(format!("{}/books/{book_id}/metadata", self.url))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaMetadata>()
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
