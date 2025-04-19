use serde::{Deserialize, Serialize};
use ureq::{Agent, Error};

pub struct StateClient;

impl StateClient {
    pub fn fetch_state(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        api_key: &str,
    ) -> Result<StateResponse, Error> {
        let result = agent
            .get(format!("{}/books/{}/state", url, book_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<StateResponse>()?;

        Ok(result)
    }

    pub fn update_state(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        tag: Option<String>,
        source: Option<String>,
        reading_status: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        let request_location = match source {
            Some(s) => Some(LocationResponse { tag, source: Some(s) }),
            None => None,
        };

        let request_statistics = StatisticsResponse {
            rating: None,
            reading_status: reading_status.to_string(),
        };

        let request = StateResponse {
            location: request_location,
            statistics: request_statistics,
        };

        agent
            .patch(format!("{}/books/{}/state", url, book_id))
            .header("api-key", api_key)
            .send_json(request)?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LocationResponse {
    pub tag: Option<String>,
    pub source: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StatisticsResponse {
    pub rating: Option<f32>,
    pub reading_status: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StateResponse {
    pub location: Option<LocationResponse>,
    pub statistics: StatisticsResponse,
}
