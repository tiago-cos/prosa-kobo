use super::models::{AuthError, JWTClaims, OAUTH_CONFIGS, OAUTH_TOKEN};
use base64::{Engine, prelude::BASE64_STANDARD};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rand::{TryRngCore, rngs::OsRng};
use serde_json::Value;
use std::{
    io::Error,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::fs;

#[rustfmt::skip]
pub async fn generate_jwt(jwt_key_path: &str, device_id: &str, duration: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs();

    let claims = JWTClaims { device_id: device_id.to_string(), exp: now + duration };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&read_jwt_secret(jwt_key_path).await),
    )
    .expect("Failed to encode token");

    BASE64_STANDARD.encode(token)
}

pub async fn verify_jwt(token: &str, jwt_key_path: &str) -> Result<String, AuthError> {
    let token = BASE64_STANDARD.decode(token).or(Err(AuthError::InvalidToken))?;
    let token = String::from_utf8(token).expect("Failed to convert token to string");
    let key = DecodingKey::from_secret(&read_jwt_secret(jwt_key_path).await);
    let validation = Validation::default();
    let token = jsonwebtoken::decode::<JWTClaims>(&token, &key, &validation)?;

    Ok(token.claims.device_id)
}

pub fn generate_oauth_config(host: &str, device_id: &str) -> Value {
    let json_string = OAUTH_CONFIGS
        .replace("{host}", host)
        .replace("{device_id}", device_id);

    serde_json::from_str(&json_string).expect("Failed to parse JSON")
}

pub fn generate_oauth_token(jwt_token: &str, jwt_duration: u64) -> Value {
    let json_string = OAUTH_TOKEN
        .replace("{jwt_token}", jwt_token)
        .replace("{jwt_duration}", &jwt_duration.to_string());

    serde_json::from_str(&json_string).expect("Failed to parse JSON")
}

pub async fn generate_jwt_secret(path: &str) -> Result<(), Error> {
    let path = Path::new(path);

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut key = [0u8; 32];
        OsRng.try_fill_bytes(&mut key).unwrap();

        fs::write(path, &key).await?;
    }

    Ok(())
}

pub async fn read_jwt_secret(path: &str) -> Vec<u8> {
    let path = Path::new(path);
    fs::read(path).await.expect("Failed to read JWT secret file")
}
