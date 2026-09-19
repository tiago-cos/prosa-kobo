use crate::{
    app::error::KoboError,
    client::prosa::{ClientError, ProsaApi},
};

pub fn download_book(client: &dyn ProsaApi, book_id: &str, api_key: &str) -> Result<Vec<u8>, KoboError> {
    let book = client.download_book(book_id, api_key)?;

    Ok(book)
}

pub fn delete_book(client: &dyn ProsaApi, book_id: &str, api_key: &str) -> Result<(), KoboError> {
    match client.delete_book(book_id, api_key) {
        Err(ClientError::NotFound) | Ok(()) => (),
        e => e?,
    }

    Ok(())
}
