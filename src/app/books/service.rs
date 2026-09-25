use crate::{
    app::{error::KoboError, kepub},
    client::prosa::{ClientError, ProsaApi},
};
use std::sync::Arc;

pub async fn download_book(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<Arc<[u8]>, KoboError> {
    kepub::get_kepub(client, book_id, api_key).await
}

pub fn delete_book(client: &dyn ProsaApi, book_id: &str, api_key: &str) -> Result<(), KoboError> {
    match client.delete_book(book_id, api_key) {
        Err(ClientError::NotFound) | Ok(()) => (),
        e => e?,
    }

    Ok(())
}
