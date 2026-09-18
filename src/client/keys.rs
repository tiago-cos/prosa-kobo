use jsonwebtoken::jwk::JwkSet;
use ureq::{Agent, Error};

pub struct KeysClient {
    pub url: String,
    pub agent: Agent,
}

impl KeysClient {
    pub fn jwks(&self) -> Result<JwkSet, Error> {
        self.agent
            .get(format!("{}/.well-known/jwks.json", self.url))
            .call()?
            .body_mut()
            .read_json::<JwkSet>()
    }
}
