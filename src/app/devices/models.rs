use serde::{Deserialize, Serialize};
use sqlx::{
    error::{DatabaseError, ErrorKind},
    prelude::FromRow,
    sqlite::SqliteError,
};
use strum_macros::{EnumMessage, EnumProperty};
type SqlxError = sqlx::Error;

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum DeviceError {
    #[strum(message = "DeviceNotFound")]
    #[strum(detailed_message = "The requested device does not exist or is not accessible.")]
    #[strum(props(StatusCode = "404"))]
    DeviceNotFound,
    #[strum(message = "DeviceAlreadyLinked")]
    #[strum(detailed_message = "This device is already linked.")]
    #[strum(props(StatusCode = "409"))]
    DeviceAlreadyLinked,
    #[strum(message = "InvalidApiKey")]
    #[strum(detailed_message = "The provided api key is invalid.")]
    #[strum(props(StatusCode = "400"))]
    InvalidApiKey,
    #[strum(message = "MissingApiKey")]
    #[strum(detailed_message = "The api key must be provided.")]
    #[strum(props(StatusCode = "400"))]
    MissingApiKey,
    #[strum(message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

#[derive(Serialize, FromRow)]
pub struct UnlinkedDevice {
    pub device_id: String,
    pub timestamp: i64,
}

#[derive(Serialize, FromRow)]
pub struct LinkedDevice {
    pub device_id: String,
    pub api_key: String,
}

impl From<SqlxError> for DeviceError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::RowNotFound => DeviceError::DeviceNotFound,
            SqlxError::Database(error) => error.downcast_ref::<SqliteError>().into(),
            _ => DeviceError::InternalError,
        }
    }
}

impl From<&SqliteError> for DeviceError {
    fn from(error: &SqliteError) -> Self {
        match error.kind() {
            ErrorKind::UniqueViolation => DeviceError::DeviceAlreadyLinked,
            _ => DeviceError::InternalError,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceAuthRequest {
    pub affiliate_name: String,
    pub app_version: String,
    pub client_key: String,
    pub device_id: String,
    pub platform_id: String,
    pub serial_number: String,
    pub user_key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceAuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: String,
    pub user_key: String,
    pub tracking_id: String,
}

impl DeviceAuthResponse {
    pub fn new(token: &str, refresh_token: &str, user_key: &str) -> Self {
        DeviceAuthResponse {
            access_token: token.to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: refresh_token.to_string(),
            user_key: user_key.to_string(),
            tracking_id: "placeholder".to_string(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RefreshTokenRequest {
    pub app_version: String,
    pub client_key: String,
    pub platform_id: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: String,
}

impl RefreshTokenResponse {
    pub fn new(token: &str, refresh_token: &str) -> Self {
        RefreshTokenResponse {
            access_token: token.to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: refresh_token.to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct LinkDeviceRequest {
    pub device_id: String,
    pub api_key: String,
}
