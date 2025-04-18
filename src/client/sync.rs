use serde::Deserialize;
use ureq::Error;

pub struct SyncClient;

impl SyncClient {
    pub fn sync_device(&self, url: &str, api_key: &str) -> Result<SyncResponse, Error> {
        ureq::get(format!("{}/sync", url))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<SyncResponse>()
    }
}

#[derive(Deserialize, Debug)]
pub struct SyncResponse {
    pub file: Vec<String>,
    pub metadata: Vec<String>,
    pub cover: Vec<String>,
    pub state: Vec<String>,
    pub deleted: Vec<String>,
}
