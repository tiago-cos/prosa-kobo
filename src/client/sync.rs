use serde::Deserialize;
use ureq::Error;

pub struct SyncClient;

impl SyncClient {
    pub fn sync_device(&self, url: &str, since: Option<i64>, api_key: &str) -> Result<SyncResponse, Error> {
        let mut request = ureq::get(format!("{}/sync", url)).header("api-key", api_key);

        if let Some(since) = since {
            request = request.query("since", since.to_string());
        }

        request.call()?.body_mut().read_json::<SyncResponse>()
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
