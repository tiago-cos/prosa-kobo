use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ureq::{Agent, Error};

pub struct ShelfClient;

impl ShelfClient {
    pub fn create_shelf(
        &self,
        url: &str,
        agent: &Agent,
        shelf_name: &str,
        owner_id: Option<String>,
        api_key: &str,
    ) -> Result<String, Error> {
        let request = ProsaShelfCreateRequest {
            name: shelf_name.to_string(),
            owner_id: owner_id,
        };

        agent
            .post(format!("{}/shelves", url))
            .header("api-key", api_key)
            .send_json(request)?
            .body_mut()
            .read_to_string()
    }

    pub fn get_shelf_metadata(
        &self,
        url: &str,
        agent: &Agent,
        shelf_id: &str,
        api_key: &str,
    ) -> Result<ProsaShelfMetadata, Error> {
        agent
            .get(format!("{}/shelves/{}", url, shelf_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaShelfMetadata>()
    }

    pub fn update_shelf_name(
        &self,
        url: &str,
        agent: &Agent,
        shelf_id: &str,
        shelf_name: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        let request = ProsaShelfUpdateRequest {
            name: shelf_name.to_string(),
        };
        agent
            .put(format!("{}/shelves/{}", url, shelf_id))
            .header("api-key", api_key)
            .send_json(request)?;

        Ok(())
    }

    pub fn delete_shelf(&self, url: &str, agent: &Agent, shelf_id: &str, api_key: &str) -> Result<(), Error> {
        agent
            .delete(format!("{}/shelves/{}", url, shelf_id))
            .header("api-key", api_key)
            .call()?;

        Ok(())
    }

    pub fn add_book_to_shelf(
        &self,
        url: &str,
        agent: &Agent,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        let request = ProsaAddBookShelfRequest {
            book_id: book_id.to_string(),
        };

        agent
            .post(format!("{}/shelves/{}/books", url, shelf_id))
            .header("api-key", api_key)
            .send_json(request)?;

        Ok(())
    }

    pub fn list_books_in_shelf(
        &self,
        url: &str,
        agent: &Agent,
        shelf_id: &str,
        api_key: &str,
    ) -> Result<Vec<String>, Error> {
        agent
            .get(format!("{}/shelves/{}/books", url, shelf_id))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<Vec<String>>()
    }

    pub fn delete_book_from_shelf(
        &self,
        url: &str,
        agent: &Agent,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        agent
            .delete(format!("{}/shelves/{}/books/{}", url, shelf_id, book_id))
            .header("api-key", api_key)
            .call()?;

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct ProsaShelfMetadata {
    pub name: String,
    pub owner_id: String,
    pub book_count: u64,
}

#[derive(Serialize, Debug)]
pub struct ProsaShelfUpdateRequest {
    pub name: String,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
pub struct ProsaShelfCreateRequest {
    pub name: String,
    pub owner_id: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ProsaAddBookShelfRequest {
    pub book_id: String,
}
