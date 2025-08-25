use crate::{
    app::error::KoboError,
    client::prosa::{Client, ClientError},
};

pub fn translate_add_shelf(client: &Client, shelf_name: &str, api_key: &str) -> Result<String, KoboError> {
    let shelf_id = client.create_shelf(shelf_name, None, api_key)?;
    Ok(shelf_id)
}

pub fn translate_add_book_to_shelf(
    client: &Client,
    shelf_id: &str,
    book_id: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    client.add_book_to_shelf(shelf_id, book_id, api_key)?;
    Ok(())
}

pub fn translate_delete_shelf(client: &Client, shelf_id: &str, api_key: &str) -> Result<(), KoboError> {
    match client.delete_shelf(shelf_id, api_key) {
        Err(ClientError::NotFound) | Ok(()) => (),
        e => e?,
    }
    Ok(())
}

pub fn translate_rename_shelf(
    client: &Client,
    shelf_id: &str,
    shelf_name: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    client.update_shelf_name(shelf_id, shelf_name, api_key)?;
    Ok(())
}

pub fn translate_delete_book_from_shelf(
    client: &Client,
    shelf_id: &str,
    book_id: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    match client.delete_book_from_shelf(shelf_id, book_id, api_key) {
        Err(ClientError::NotFound) | Ok(()) => (),
        e => e?,
    }
    Ok(())
}
