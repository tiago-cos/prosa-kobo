use sqlx::FromRow;
use strum_macros::{EnumMessage, EnumProperty};

type SqlxError = sqlx::Error;

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum BookTokenError {
    #[strum(message = "InvalidToken")]
    #[strum(detailed_message = "The provided book token is invalid.")]
    #[strum(props(StatusCode = "403"))]
    InvalidToken,
    #[strum(message = "Internal error")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

#[derive(FromRow)]
pub struct BookToken {
    pub book_id: String,
    pub expiration: i64,
    pub api_key: String,
}

impl From<SqlxError> for BookTokenError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::RowNotFound => BookTokenError::InvalidToken,
            _ => BookTokenError::InternalError,
        }
    }
}

pub const BOOK_TOKEN_SIZE: usize = 128;
