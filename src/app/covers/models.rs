use crate::app::error::unmapped;
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
