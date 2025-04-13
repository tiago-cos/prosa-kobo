use super::models::{AuthError, JWTClaims};
use base64::{prelude::BASE64_STANDARD, Engine};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

#[rustfmt::skip]
pub async fn generate_jwt(secret: &str, device_id: &str, duration: &u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch")
        .as_secs();

    let claims = JWTClaims { device_id: device_id.to_string(), exp: now + duration };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Failed to encode token");

    BASE64_STANDARD.encode(token)
}

pub async fn verify_jwt(token: &str, secret: &str) -> Result<String, AuthError> {
    let token = BASE64_STANDARD.decode(token).or(Err(AuthError::InvalidToken))?;
    let token = String::from_utf8(token).expect("Failed to convert token to string");
    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();
    let token = jsonwebtoken::decode::<JWTClaims>(&token, &key, &validation)?;

    Ok(token.claims.device_id)
}
