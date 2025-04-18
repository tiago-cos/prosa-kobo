use serde::Deserialize;
use ureq::Error;

pub struct MetadataClient;

impl MetadataClient {
    pub fn fetch_metadata(&self, url: &str, book_id: &str, api_key: &str) -> Result<MetadataResponse, Error> {
        ureq::get(format!("{}/books/{}/metadata", url, book_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<MetadataResponse>()
    }

    pub fn fetch_size(&self, url: &str, book_id: &str, api_key: &str) -> Result<u64, Error> {
        ureq::get(format!("{}/books/{}/size", url, book_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<u64>()
    }
}

#[derive(Deserialize, Debug)]
pub struct ContributorResponse {
    pub name: String,
    pub role: String,
}

#[derive(Deserialize, Debug)]
pub struct SeriesResponse {
    pub title: String,
    pub number: f32,
}

#[derive(Deserialize, Debug)]
pub struct MetadataResponse {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub publication_date: Option<i64>,
    pub isbn: Option<String>,
    pub contributors: Option<Vec<ContributorResponse>>,
    pub genres: Option<Vec<String>>,
    pub series: Option<SeriesResponse>,
    pub page_count: Option<i64>,
    pub language: Option<String>,
}
