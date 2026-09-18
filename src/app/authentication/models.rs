use serde::{Deserialize, Serialize};
use strum_macros::{EnumMessage, EnumProperty};

type JwtError = jsonwebtoken::errors::Error;
type JwtErrorKind = jsonwebtoken::errors::ErrorKind;

#[rustfmt::skip]
#[derive(EnumMessage, EnumProperty, Debug, PartialEq, Eq)]
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
    #[strum(message = "MissingDeviceId", detailed_message = "No device id was provided.")]
    #[strum(props(StatusCode = "401"))]
    MissingDeviceId,
    #[strum(message = "UnauthenticatedDevice", detailed_message = "Device is recognized, but is unauthenticated.")]
    #[strum(props(StatusCode = "401"))]
    UnauthenticatedDevice,
    #[strum(message = "InternalError", detailed_message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

impl From<JwtError> for AuthError {
    fn from(err: JwtError) -> Self {
        match err.kind() {
            JwtErrorKind::ExpiredSignature => AuthError::ExpiredToken,
            JwtErrorKind::InvalidToken | JwtErrorKind::InvalidIssuer | JwtErrorKind::InvalidAlgorithm => {
                AuthError::InvalidToken
            }
            JwtErrorKind::InvalidSignature => AuthError::InvalidSignature,
            _ => AuthError::InternalError,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JWTClaims {
    pub device_id: String,
    pub exp: u64,
}

#[derive(Clone)]
pub struct AuthToken {
    pub device_id: String,
    pub api_key: String,
}

pub const PROSA_ISSUER: &str = "prosa";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProsaRole {
    Admin(String),
    User(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProsaJWTClaims {
    pub role: ProsaRole,
    pub capabilities: Vec<String>,
    pub exp: u64,
    pub session_id: String,
    pub iss: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProsaToken {
    pub user_id: String,
    pub is_admin: bool,
    pub capabilities: Vec<String>,
    pub session_id: String,
}

impl From<ProsaJWTClaims> for ProsaToken {
    fn from(claims: ProsaJWTClaims) -> Self {
        let (user_id, is_admin) = match claims.role {
            ProsaRole::Admin(user_id) => (user_id, true),
            ProsaRole::User(user_id) => (user_id, false),
        };

        Self {
            user_id,
            is_admin,
            capabilities: claims.capabilities,
            session_id: claims.session_id,
        }
    }
}

pub const OAUTH_CONFIGS: &str = r#"{ "token_endpoint": "{host}/oauth/connect/token?device_id={device_id}" }"#;

pub const OAUTH_TOKEN: &str = r#"
{
  "id_token": "{jwt_token}",
  "access_token": "{jwt_token}",
  "expires_in": {jwt_duration},
  "token_type": "Bearer",
  "refresh_token": "{jwt_token}",
  "scope": "openid profile kobo_profile public_api_authenticated public_api_anonymous offline_access"
}
"#;
