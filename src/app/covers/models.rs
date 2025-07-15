use sqlx::FromRow;
use strum_macros::{EnumMessage, EnumProperty};

type SqlxError = sqlx::Error;

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum CoverTokenError {
    #[strum(message = "InvalidToken")]
    #[strum(detailed_message = "The provided cover token is invalid.")]
    #[strum(props(StatusCode = "403"))]
    InvalidToken,
    #[strum(message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

#[derive(FromRow)]
pub struct CoverToken {
    pub book_id: String,
    pub expiration: i64,
    pub api_key: String,
}

impl From<SqlxError> for CoverTokenError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::RowNotFound => CoverTokenError::InvalidToken,
            _ => CoverTokenError::InternalError,
        }
    }
}

pub const COVER_TOKEN_SIZE: usize = 128;
