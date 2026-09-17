use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ureq::{Agent, Error};

pub struct StateClient {
    pub url: String,
    pub agent: Agent,
}

impl StateClient {
    pub fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, Error> {
        self.agent
            .get(format!("{}/books/{book_id}/state", self.url))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaState>()
    }

    pub fn patch_state(
        &self,
        book_id: &str,
        location: Option<&str>,
        reading_status: ProsaReadingStatus,
        api_key: &str,
    ) -> Result<(), Error> {
        let request = ProsaStatePatch {
            location,
            statistics: ProsaStatisticsPatch {
                rating: None,
                reading_status: Some(reading_status),
            },
        };

        self.agent
            .patch(format!("{}/books/{book_id}/state", self.url))
            .header("api-key", api_key)
            .send_json(request)?;

        Ok(())
    }

    pub fn replace_state(&self, book_id: &str, state: &ProsaState, api_key: &str) -> Result<(), Error> {
        self.agent
            .put(format!("{}/books/{book_id}/state", self.url))
            .header("api-key", api_key)
            .send_json(state)?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProsaReadingStatus {
    Unread,
    Reading,
    Read,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ProsaState {
    pub location: Option<String>,
    pub statistics: ProsaStatistics,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ProsaStatistics {
    pub rating: Option<f32>,
    pub reading_status: ProsaReadingStatus,
}

/// `PATCH` leaves out what it does not change, so every field is optional and
/// an absent one must not reach the wire as `null`.
#[skip_serializing_none]
#[derive(Serialize, Debug)]
struct ProsaStatePatch<'a> {
    location: Option<&'a str>,
    statistics: ProsaStatisticsPatch,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
struct ProsaStatisticsPatch {
    rating: Option<f32>,
    reading_status: Option<ProsaReadingStatus>,
}
