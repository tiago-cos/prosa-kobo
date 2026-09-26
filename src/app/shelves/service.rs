use super::models::{CreateShelfRequest, ShelfItem};
use crate::{
    app::error::KoboError,
    client::prosa::{ClientError, ProsaApi},
};

pub async fn translate_create_shelf(
    client: &dyn ProsaApi,
    request: &CreateShelfRequest,
    api_key: &str,
) -> Result<String, KoboError> {
    let shelf_id = translate_add_shelf(client, &request.name, api_key).await?;
    translate_add_books_to_shelf(client, &shelf_id, &request.items, api_key).await?;

    Ok(shelf_id)
}

pub async fn translate_add_books_to_shelf(
    client: &dyn ProsaApi,
    shelf_id: &str,
    items: &[ShelfItem],
    api_key: &str,
) -> Result<Vec<String>, KoboError> {
    for item in items {
        translate_add_book_to_shelf(client, shelf_id, &item.revision_id, api_key).await?;
    }

    Ok(items.iter().map(|item| item.revision_id.clone()).collect())
}

pub async fn translate_delete_books_from_shelf(
    client: &dyn ProsaApi,
    shelf_id: &str,
    items: &[ShelfItem],
    api_key: &str,
) -> Result<(), KoboError> {
    for item in items {
        translate_delete_book_from_shelf(client, shelf_id, &item.revision_id, api_key).await?;
    }

    Ok(())
}

async fn translate_add_shelf(
    client: &dyn ProsaApi,
    shelf_name: &str,
    api_key: &str,
) -> Result<String, KoboError> {
    match client.create_shelf(shelf_name, None, None, api_key).await {
        Err(ClientError::Conflict) => find_shelf(client, shelf_name, api_key)
            .await?
            .ok_or_else(|| ClientError::Conflict.into()),
        result => Ok(result?),
    }
}

async fn find_shelf(
    client: &dyn ProsaApi,
    shelf_name: &str,
    api_key: &str,
) -> Result<Option<String>, KoboError> {
    let username = client.identity(api_key).await?.username;

    for shelf_id in client.search_shelves(&username, shelf_name, api_key).await? {
        if client.get_shelf_metadata(&shelf_id, api_key).await?.name == shelf_name {
            return Ok(Some(shelf_id));
        }
    }

    Ok(None)
}

async fn translate_add_book_to_shelf(
    client: &dyn ProsaApi,
    shelf_id: &str,
    book_id: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    match client.add_book_to_shelf(shelf_id, book_id, api_key).await {
        Err(ClientError::Conflict) | Ok(()) => (),
        e => e?,
    }
    Ok(())
}

pub async fn translate_delete_shelf(
    client: &dyn ProsaApi,
    shelf_id: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    match client.delete_shelf(shelf_id, api_key).await {
        Err(ClientError::NotFound) | Ok(()) => (),
        e => e?,
    }
    Ok(())
}

pub async fn translate_rename_shelf(
    client: &dyn ProsaApi,
    shelf_id: &str,
    shelf_name: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    client.update_shelf_name(shelf_id, shelf_name, api_key).await?;
    Ok(())
}

async fn translate_delete_book_from_shelf(
    client: &dyn ProsaApi,
    shelf_id: &str,
    book_id: &str,
    api_key: &str,
) -> Result<(), KoboError> {
    match client.delete_book_from_shelf(shelf_id, book_id, api_key).await {
        Err(ClientError::NotFound) | Ok(()) => (),
        e => e?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::mock::{MockProsaClient, ProsaMethod};

    #[tokio::test]
    async fn adding_a_book_the_shelf_already_holds_is_not_an_error() {
        let client = MockProsaClient::new();
        client
            .seed_book("book")
            .seed_shelf("shelf", "Favourites", &["book"]);

        translate_add_book_to_shelf(&client, "shelf", "book", "key")
            .await
            .expect("Expected the conflict to be swallowed");

        assert_eq!(client.stored_shelf_books("shelf"), vec!["book".to_owned()]);
    }

    #[tokio::test]
    async fn deleting_a_shelf_that_is_already_gone_is_not_an_error() {
        let client = MockProsaClient::new();

        translate_delete_shelf(&client, "shelf", "key")
            .await
            .expect("Expected the missing shelf to be swallowed");

        assert_eq!(client.call_count(ProsaMethod::DeleteShelf), 1);
    }

    #[tokio::test]
    async fn renaming_a_missing_shelf_still_fails() {
        let client = MockProsaClient::new();

        let result = translate_rename_shelf(&client, "shelf", "Favourites", "key").await;

        assert!(result.is_err());
    }
}
