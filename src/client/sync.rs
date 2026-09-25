use reqwest::{Client, Error};
use serde::Deserialize;

pub struct SyncClient {
    pub url: String,
    pub http: Client,
}

impl SyncClient {
    pub async fn sync_device(&self, sync_token: Option<i64>, api_key: &str) -> Result<ProsaSync, Error> {
        let mut request = self
            .http
            .get(format!("{}/sync", self.url))
            .header("api-key", api_key);

        if let Some(sync_token) = sync_token {
            request = request.query(&[("sync_token", sync_token.to_string())]);
        }

        request
            .send()
            .await?
            .error_for_status()?
            .json::<ProsaSync>()
            .await
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ProsaSync {
    pub new_sync_token: i64,
    pub unsynced_books: ProsaBookSync,
    pub unsynced_shelves: ProsaShelfSync,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ProsaBookSync {
    pub file: Vec<String>,
    pub metadata: Vec<String>,
    pub cover: Vec<String>,
    pub state: Vec<String>,
    pub annotations: Vec<String>,
    pub deleted: Vec<String>,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ProsaShelfSync {
    pub metadata: Vec<String>,
    pub contents: Vec<String>,
    pub deleted: Vec<String>,
}
