use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ureq::{Agent, Error};

pub struct ShelfClient {
    pub url: String,
    pub agent: Agent,
}

impl ShelfClient {
    pub fn create_shelf(
        &self,
        shelf_name: &str,
        owner_id: Option<&str>,
        shelf_id: Option<&str>,
        api_key: &str,
    ) -> Result<String, Error> {
        let request = ProsaShelfCreateRequest {
            name: shelf_name,
            owner_id,
            shelf_id,
        };

        self.agent
            .post(format!("{}/shelves", self.url))
            .header("api-key", api_key)
            .send_json(request)?
            .body_mut()
            .read_to_string()
    }

    pub fn get_shelf_metadata(&self, shelf_id: &str, api_key: &str) -> Result<ProsaShelfMetadata, Error> {
        self.agent
            .get(format!("{}/shelves/{shelf_id}", self.url))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<ProsaShelfMetadata>()
    }

    pub fn update_shelf_name(&self, shelf_id: &str, shelf_name: &str, api_key: &str) -> Result<(), Error> {
        let request = ProsaShelfUpdateRequest { name: shelf_name };

        self.agent
            .put(format!("{}/shelves/{shelf_id}", self.url))
            .header("api-key", api_key)
            .send_json(request)?;

        Ok(())
    }

    pub fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), Error> {
        self.agent
            .delete(format!("{}/shelves/{shelf_id}", self.url))
            .header("api-key", api_key)
            .call()?;

        Ok(())
    }

    pub fn add_book_to_shelf(&self, shelf_id: &str, book_id: &str, api_key: &str) -> Result<(), Error> {
        let request = ProsaAddBookShelfRequest { book_id };

        self.agent
            .post(format!("{}/shelves/{shelf_id}/books", self.url))
            .header("api-key", api_key)
            .send_json(request)?;

        Ok(())
    }

    pub fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, Error> {
        self.agent
            .get(format!("{}/shelves/{shelf_id}/books", self.url))
            .header("api-key", api_key)
            .call()?
            .body_mut()
            .read_json::<Vec<String>>()
    }

    pub fn delete_book_from_shelf(&self, shelf_id: &str, book_id: &str, api_key: &str) -> Result<(), Error> {
        self.agent
            .delete(format!("{}/shelves/{shelf_id}/books/{book_id}", self.url))
            .header("api-key", api_key)
            .call()?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaShelfMetadata {
    pub name: String,
    pub owner_id: String,
    pub book_count: u64,
}

#[derive(Serialize, Debug)]
struct ProsaShelfUpdateRequest<'a> {
    name: &'a str,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
struct ProsaShelfCreateRequest<'a> {
    name: &'a str,
    owner_id: Option<&'a str>,
    shelf_id: Option<&'a str>,
}

#[derive(Serialize, Debug)]
struct ProsaAddBookShelfRequest<'a> {
    book_id: &'a str,
}
