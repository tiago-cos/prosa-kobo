use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt::Debug;
use std::str::FromStr;
use strum::{EnumMessage, EnumProperty};

pub trait KoboErrorTrait: EnumMessage + EnumProperty + Debug {}
impl<T> KoboErrorTrait for T where T: EnumMessage + EnumProperty + Debug {}
pub type KoboError = Box<dyn KoboErrorTrait>;

impl<T> From<T> for KoboError
where
    T: KoboErrorTrait + 'static,
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

impl IntoResponse for KoboError {
    fn into_response(self) -> Response {
        let message = self.get_message().expect("Failed to extract message from error");

        let detailed_message = self
            .get_detailed_message()
            .expect("Falied to get detailed message from error");

        let status_code = self
            .get_str("StatusCode")
            .expect("Failed to extract status code from error");

        let status_code = StatusCode::from_str(&status_code).expect("Failed to parse status code from error");

        let response = ErrorResponse {
            error_code: message.to_string(),
            message: detailed_message.to_string(),
        };

        (status_code, Json(response)).into_response()
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
}
