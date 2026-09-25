use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

const SEARCH_PAGE_SIZE: u64 = 100;

pub struct ShelfClient {
    pub url: String,
    pub http: Client,
}

impl ShelfClient {
    pub async fn create_shelf(
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

        self.http
            .post(format!("{}/shelves", self.url))
            .header("api-key", api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn search_shelves(
        &self,
        username: &str,
        name: &str,
        page: u64,
        api_key: &str,
    ) -> Result<ProsaShelfSearch, Error> {
        self.http
            .get(format!("{}/shelves", self.url))
            .header("api-key", api_key)
            .query(&[("username", username)])
            .query(&[("name", name)])
            .query(&[("page", page.to_string())])
            .query(&[("size", SEARCH_PAGE_SIZE.to_string())])
            .send()
            .await?
            .error_for_status()?
            .json::<ProsaShelfSearch>()
            .await
    }

    pub async fn get_shelf_metadata(
        &self,
        shelf_id: &str,
        api_key: &str,
    ) -> Result<ProsaShelfMetadata, Error> {
        self.http
            .get(format!("{}/shelves/{shelf_id}", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<ProsaShelfMetadata>()
            .await
    }

    pub async fn update_shelf_name(
        &self,
        shelf_id: &str,
        shelf_name: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        let request = ProsaShelfUpdateRequest { name: shelf_name };

        self.http
            .put(format!("{}/shelves/{shelf_id}", self.url))
            .header("api-key", api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), Error> {
        self.http
            .delete(format!("{}/shelves/{shelf_id}", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn add_book_to_shelf(&self, shelf_id: &str, book_id: &str, api_key: &str) -> Result<(), Error> {
        let request = ProsaAddBookShelfRequest { book_id };

        self.http
            .post(format!("{}/shelves/{shelf_id}/books", self.url))
            .header("api-key", api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, Error> {
        self.http
            .get(format!("{}/shelves/{shelf_id}/books", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<String>>()
            .await
    }

    pub async fn delete_book_from_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        self.http
            .delete(format!("{}/shelves/{shelf_id}/books/{book_id}", self.url))
            .header("api-key", api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaShelfMetadata {
    pub name: String,
    pub owner_id: String,
    pub book_count: u64,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProsaShelfSearch {
    pub shelf_ids: Vec<String>,
    pub total_pages: u64,
    pub current_page: u64,
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
