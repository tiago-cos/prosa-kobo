use serde::{Deserialize, Serialize};
use strum_macros::{EnumMessage, EnumProperty};

type JwtError = jsonwebtoken::errors::Error;
type JwtErrorKind = jsonwebtoken::errors::ErrorKind;

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
    #[strum(
        message = "NotLinked",
        detailed_message = "This device is not associated with an API key."
    )]
    #[strum(props(StatusCode = "403"))]
    NotLinked,
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
