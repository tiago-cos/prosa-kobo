use crate::{
    app::{annotations, error::KoboError, kepub},
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

pub async fn delete_book(client: &dyn ProsaApi, book_id: &str, api_key: &str) -> Result<(), KoboError> {
    match client.delete_book(book_id, api_key).await {
        Err(ClientError::NotFound) | Ok(()) => (),
        e => e?,
    }

    annotations::service::delete_etag(book_id).await?;
    kepub::evict(book_id);

    Ok(())
}
