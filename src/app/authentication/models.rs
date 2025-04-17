use serde::{Deserialize, Serialize};
use strum_macros::{EnumMessage, EnumProperty};

type JwtError = jsonwebtoken::errors::Error;
type JwtErrorKind = jsonwebtoken::errors::ErrorKind;

#[rustfmt::skip]
#[derive(EnumMessage, EnumProperty, Debug)]
pub enum AuthError {
    #[strum(message = "ExpiredToken", detailed_message = "Expired token")]
    #[strum(props(StatusCode = "401"))]
    ExpiredToken,
    #[strum(message = "InvalidToken", detailed_message = "Invalid token")]
    #[strum(props(StatusCode = "401"))]
    InvalidToken,
    #[strum(message = "InvalidSignature", detailed_message = "Invalid signature")]
    #[strum(props(StatusCode = "401"))]
    InvalidSignature,
    #[strum(message = "InvalidHeader", detailed_message = "Invalid authentication header")]
    #[strum(props(StatusCode = "400"))]
    InvalidAuthHeader,
    #[strum(message = "MissingAuth", detailed_message = "No authentication was provided.")]
    #[strum(props(StatusCode = "401"))]
    MissingAuth,
    #[strum(message = "InternalError", detailed_message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

impl From<JwtError> for AuthError {
    fn from(err: JwtError) -> Self {
        match err.kind() {
            JwtErrorKind::ExpiredSignature => AuthError::ExpiredToken,
            JwtErrorKind::InvalidToken => AuthError::InvalidToken,
            JwtErrorKind::InvalidSignature => AuthError::InvalidSignature,
            _ => AuthError::InternalError,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JWTClaims {
    pub device_id: String,
    pub exp: u64,
}

#[derive(Clone)]
pub struct AuthToken {
    pub device_id: String,
    pub api_key: String,
}

//TODO add schema option
pub const OAUTH_CONFIGS: &str =
    r#"{ "token_endpoint": "http://{host}/oauth/connect/token?device_id={device_id}" }"#;

//TODO put {jwt_duration} in expires_in
pub const OAUTH_TOKEN: &str = r#"
{
  "id_token": "{jwt_token}",
  "access_token": "{jwt_token}",
  "expires_in": 60,
  "token_type": "Bearer",
  "refresh_token": "{jwt_token}",
  "scope": "openid profile kobo_profile public_api_authenticated public_api_anonymous offline_access"
}
"#;
