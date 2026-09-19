use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, sqlite::SqliteError};
use strum_macros::{EnumMessage, EnumProperty};

type SqlxError = sqlx::Error;

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum DeviceError {
    #[strum(message = "DeviceNotFound")]
    #[strum(detailed_message = "The requested device does not exist or is not accessible.")]
    #[strum(props(StatusCode = "404"))]
    DeviceNotFound,
    #[strum(message = "InvalidDeviceName")]
    #[strum(detailed_message = "A device name must be provided.")]
    #[strum(props(StatusCode = "400"))]
    InvalidDeviceName,
    #[strum(message = "InvalidApiKey")]
    #[strum(detailed_message = "The provided api key is invalid.")]
    #[strum(props(StatusCode = "400"))]
    InvalidApiKey,
    #[strum(message = "InsufficientApiKey")]
    #[strum(detailed_message = "The provided api key does not grant read access.")]
    #[strum(props(StatusCode = "400"))]
    InsufficientApiKey,
    #[strum(message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

#[derive(FromRow)]
pub struct LinkedDevice {
    pub device_id: String,
    pub lookup_key: String,
    pub user_id: String,
    pub name: String,
    pub api_key: String,
}

#[derive(Serialize)]
pub struct LinkedDeviceResponse {
    pub device_id: String,
    pub user_id: String,
    pub name: String,
}

impl From<LinkedDevice> for LinkedDeviceResponse {
    fn from(device: LinkedDevice) -> Self {
        Self {
            device_id: device.device_id,
            user_id: device.user_id,
            name: device.name,
        }
    }
}

#[derive(Serialize)]
pub struct LinkDeviceResponse {
    pub device_id: String,
    pub api_endpoint: String,
}

#[derive(Deserialize)]
pub struct LinkDeviceRequest {
    pub user_id: Option<String>,
    pub name: String,
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct ListLinkedDevicesQuery {
    pub user_id: Option<String>,
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
    fn from(_: &SqliteError) -> Self {
        DeviceError::InternalError
    }
}
