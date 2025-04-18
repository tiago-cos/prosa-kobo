use sqlx::FromRow;
use strum_macros::{EnumMessage, EnumProperty};

type SqlxError = sqlx::Error;

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum TokenError {
    #[strum(message = "InvalidToken")]
    #[strum(detailed_message = "The provided token is invalid.")]
    #[strum(props(StatusCode = "403"))]
    InvalidToken,
    #[strum(message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

#[derive(FromRow)]
pub struct DownloadToken {
    pub token: String,
    pub expiration: i64,
}

impl From<SqlxError> for TokenError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::RowNotFound => TokenError::InvalidToken,
            _ => TokenError::InternalError,
        }
    }
}

pub const TOKEN_SIZE: usize = 128;
