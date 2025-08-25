use serde::Deserialize;
use ureq::{Agent, Error};

pub struct SyncClient {
    pub url: String,
    pub agent: Agent,
}

impl SyncClient {
    pub fn sync_device(&self, since: Option<i64>, api_key: &str) -> Result<ProsaSync, Error> {
        let mut request = self
            .agent
            .get(format!("{}/sync", self.url))
            .header("api-key", api_key);

        if let Some(since) = since {
            request = request.query("since", since.to_string());
        }

        request.call()?.body_mut().read_json::<ProsaSync>()
    }
}

#[derive(Deserialize, Debug)]
pub struct ProsaSync {
    pub book: ProsaBookSync,
    pub shelf: ProsaShelfSync,
}

#[derive(Deserialize, Debug)]
pub struct ProsaBookSync {
    pub file: Vec<String>,
    pub metadata: Vec<String>,
    pub cover: Vec<String>,
    pub state: Vec<String>,
    pub annotations: Vec<String>,
    pub deleted: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct ProsaShelfSync {
    pub metadata: Vec<String>,
    pub contents: Vec<String>,
    pub deleted: Vec<String>,
}
