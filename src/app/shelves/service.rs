use crate::{
    app::error::KoboError,
    client::prosa::{ClientError, ProsaApi},
};

pub fn translate_add_shelf(
    client: &dyn ProsaApi,
    shelf_name: &str,
    api_key: &str,
) -> Result<String, KoboError> {
    match client.create_shelf(shelf_name, None, None, api_key) {
        Err(ClientError::Conflict) => {
            find_shelf(client, shelf_name, api_key)?.ok_or_else(|| ClientError::Conflict.into())
        }
        result => Ok(result?),
    }
}

fn find_shelf(client: &dyn ProsaApi, shelf_name: &str, api_key: &str) -> Result<Option<String>, KoboError> {
    let username = client.identity(api_key)?.username;

    for shelf_id in client.search_shelves(&username, shelf_name, api_key)? {
        if client.get_shelf_metadata(&shelf_id, api_key)?.name == shelf_name {
            return Ok(Some(shelf_id));
        }
    }

    Ok(None)
}

pub fn translate_add_book_to_shelf(
    client: &dyn ProsaApi,
    shelf_id: &str,
    book_id: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    match client.add_book_to_shelf(shelf_id, book_id, api_key) {
        Err(ClientError::Conflict) | Ok(()) => (),
        e => e?,
    }
    Ok(())
}

pub fn translate_delete_shelf(client: &dyn ProsaApi, shelf_id: &str, api_key: &str) -> Result<(), KoboError> {
    match client.delete_shelf(shelf_id, api_key) {
        Err(ClientError::NotFound) | Ok(()) => (),
        e => e?,
    }
    Ok(())
}

pub fn translate_rename_shelf(
    client: &dyn ProsaApi,
    shelf_id: &str,
    shelf_name: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    client.update_shelf_name(shelf_id, shelf_name, api_key)?;
    Ok(())
}

pub fn translate_delete_book_from_shelf(
    client: &dyn ProsaApi,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::mock::{MockProsaClient, ProsaMethod};

    #[test]
    fn adding_a_book_the_shelf_already_holds_is_not_an_error() {
        let client = MockProsaClient::new();
        client
            .seed_book("book")
            .seed_shelf("shelf", "Favourites", &["book"]);

        translate_add_book_to_shelf(&client, "shelf", "book", "key")
            .expect("Expected the conflict to be swallowed");

        assert_eq!(client.stored_shelf_books("shelf"), vec!["book".to_owned()]);
    }

    #[test]
    fn deleting_a_shelf_that_is_already_gone_is_not_an_error() {
        let client = MockProsaClient::new();

        translate_delete_shelf(&client, "shelf", "key").expect("Expected the missing shelf to be swallowed");

        assert_eq!(client.call_count(ProsaMethod::DeleteShelf), 1);
    }

    #[test]
    fn renaming_a_missing_shelf_still_fails() {
        let client = MockProsaClient::new();

        let result = translate_rename_shelf(&client, "shelf", "Favourites", "key");

        assert!(result.is_err());
    }
}
