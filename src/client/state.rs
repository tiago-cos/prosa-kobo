use serde::Deserialize;
use ureq::Error;

pub struct StateClient;

impl StateClient {
    pub fn fetch_state(&self, url: &str, book_id: &str, api_key: &str) -> Result<StateResponse, Error> {
        let mut result = ureq::get(format!("{}/books/{}/state", url, book_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<StateResponse>()?;

        result.statistics.reading_status = match result.statistics.reading_status.as_ref() {
            "Read" => "Finished".to_string(),
            status => status.to_string(),
        };

        Ok(result)
    }
}

#[derive(Deserialize, Debug)]
pub struct LocationResponse {
    pub tag: Option<String>,
    pub source: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct StatisticsResponse {
    pub rating: Option<f32>,
    pub reading_status: String,
}

#[derive(Deserialize, Debug)]
pub struct StateResponse {
    pub location: Option<LocationResponse>,
    pub statistics: StatisticsResponse,
}
