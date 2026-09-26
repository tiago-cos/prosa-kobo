use crate::app::error::unmapped;
use serde::Deserialize;
use strum_macros::{EnumMessage, EnumProperty};

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum CoverError {
    #[strum(message = "InternalError")]
    #[strum(detailed_message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

impl From<sqlx::Error> for CoverError {
    fn from(error: sqlx::Error) -> Self {
        unmapped(&error, CoverError::InternalError)
    }
}

#[derive(Deserialize)]
pub struct CoverSize {
    pub width: Option<u32>,
    pub height: Option<u32>,
}
