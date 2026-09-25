use jsonwebtoken::jwk::JwkSet;
use reqwest::{Client, Error};

pub struct KeysClient {
    pub url: String,
    pub http: Client,
}

impl KeysClient {
    pub async fn jwks(&self) -> Result<JwkSet, Error> {
        self.http
            .get(format!("{}/.well-known/jwks.json", self.url))
            .send()
            .await?
            .error_for_status()?
            .json::<JwkSet>()
            .await
    }
}
