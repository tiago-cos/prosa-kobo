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
    ) -> Result<ProsaState, Error> {
        let result = agent
            .get(format!("{}/books/{}/state", url, book_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaState>()?;

        Ok(result)
    }

    pub fn patch_state(
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
            Some(s) => Some(ProsaLocation { tag, source: Some(s) }),
            None => None,
        };

        let request_statistics = ProsaStatistics {
            rating: None,
            reading_status: reading_status.to_string(),
        };

        let request = ProsaState {
            location: request_location,
            statistics: request_statistics,
        };

        agent
            .patch(format!("{}/books/{}/state", url, book_id))
            .header("api-key", api_key)
            .send_json(request)?;

        Ok(())
    }

    pub fn update_rating(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        rating: u8,
        api_key: &str,
    ) -> Result<(), Error> {
        let mut previous_state = self.fetch_state(url, agent, book_id, api_key)?;

        previous_state.statistics.rating = match rating {
            0 => None,
            r => Some(r.into()),
        };

        agent
            .put(format!("{}/books/{}/state", url, book_id))
            .header("api-key", api_key)
            .send_json(previous_state)?;

        Ok(())
    }

    pub fn fetch_rating(
        &self,
        url: &str,
        agent: &Agent,
        book_id: &str,
        api_key: &str,
    ) -> Result<Option<u8>, Error> {
        let state = self.fetch_state(url, agent, book_id, api_key)?;
        let rating = state.statistics.rating.map(|s| s.round() as u8);
        Ok(rating)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProsaLocation {
    pub tag: Option<String>,
    pub source: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProsaStatistics {
    pub rating: Option<f32>,
    pub reading_status: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProsaState {
    pub location: Option<ProsaLocation>,
    pub statistics: ProsaStatistics,
}
