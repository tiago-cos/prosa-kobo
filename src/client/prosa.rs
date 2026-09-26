use super::{
    annotations::{ProsaAnnotation, ProsaAnnotationPatch, ProsaAnnotationRequest},
    book::ProsaBookFileMetadata,
    health::ProsaHealth,
    identity::ProsaIdentity,
    metadata::ProsaMetadata,
    shelf::{
        ProsaAddBookShelfRequest, ProsaShelfCreateRequest, ProsaShelfMetadata, ProsaShelfSearch,
        ProsaShelfUpdateRequest,
    },
    state::{ProsaReadingStatus, ProsaState, ProsaStatePatch, ProsaStatisticsPatch},
    sync::ProsaSync,
};
use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;
use reqwest::{Error, Method, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use strum_macros::{EnumMessage, EnumProperty};

const MAX_BOOK_SIZE: usize = 50 * 1024 * 1024;
const MAX_COVER_SIZE: usize = 10 * 1024 * 1024;
const SEARCH_PAGE_SIZE: u64 = 100;

#[derive(EnumMessage, EnumProperty, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientError {
    #[strum(message = "BadRequest")]
    #[strum(detailed_message = "Bad client request.")]
    #[strum(props(StatusCode = "400"))]
    BadRequest,
    #[strum(message = "Unauthorized")]
    #[strum(detailed_message = "Unauthorized client request.")]
    #[strum(props(StatusCode = "401"))]
    Unauthorized,
    #[strum(message = "Forbidden")]
    #[strum(detailed_message = "Forbidden client request.")]
    #[strum(props(StatusCode = "403"))]
    Forbidden,
    #[strum(message = "NotFound")]
    #[strum(detailed_message = "Client request not found.")]
    #[strum(props(StatusCode = "404"))]
    NotFound,
    #[strum(message = "Conflict")]
    #[strum(detailed_message = "Conflicting client request.")]
    #[strum(props(StatusCode = "409"))]
    Conflict,
    #[strum(message = "InternalError")]
    #[strum(detailed_message = "Client internal error.")]
    #[strum(props(StatusCode = "500"))]
    InternalError,
}

#[async_trait]
pub trait ProsaApi: Send + Sync {
    async fn health(&self) -> Result<ProsaHealth, ClientError>;

    async fn jwks(&self) -> Result<JwkSet, ClientError>;

    async fn identity(&self, api_key: &str) -> Result<ProsaIdentity, ClientError>;

    async fn sync_device(&self, sync_token: Option<i64>, api_key: &str) -> Result<ProsaSync, ClientError>;

    async fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, ClientError>;

    async fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, ClientError>;

    async fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError>;

    async fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), ClientError>;

    async fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError>;

    async fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, ClientError>;

    async fn patch_state(
        &self,
        book_id: &str,
        location: Option<&str>,
        reading_status: ProsaReadingStatus,
        api_key: &str,
    ) -> Result<(), ClientError>;

    async fn replace_state(
        &self,
        book_id: &str,
        state: &ProsaState,
        api_key: &str,
    ) -> Result<(), ClientError>;

    async fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, ClientError>;

    async fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, ClientError>;

    async fn add_annotation(
        &self,
        book_id: &str,
        annotation: &ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, ClientError>;

    async fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), ClientError>;

    async fn delete_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError>;

    async fn create_shelf(
        &self,
        shelf_name: &str,
        owner_id: Option<&str>,
        shelf_id: Option<&str>,
        api_key: &str,
    ) -> Result<String, ClientError>;

    async fn search_shelves(
        &self,
        username: &str,
        name: &str,
        api_key: &str,
    ) -> Result<Vec<String>, ClientError>;

    async fn get_shelf_metadata(
        &self,
        shelf_id: &str,
        api_key: &str,
    ) -> Result<ProsaShelfMetadata, ClientError>;

    async fn update_shelf_name(
        &self,
        shelf_id: &str,
        shelf_name: &str,
        api_key: &str,
    ) -> Result<(), ClientError>;

    async fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), ClientError>;

    async fn add_book_to_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError>;

    async fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, ClientError>;

    async fn delete_book_from_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError>;
}

pub struct Client {
    url: String,
    http: reqwest::Client,
}

impl Client {
    pub fn new(scheme: &str, host: &str, port: u16) -> Self {
        Client {
            url: format!("{scheme}://{host}:{port}"),
            http: reqwest::Client::new(),
        }
    }

    fn request(&self, method: Method, path: &str, api_key: &str) -> RequestBuilder {
        self.http
            .request(method, format!("{}{path}", self.url))
            .header("api-key", api_key)
    }
}

async fn send(request: RequestBuilder) -> Result<Response, ClientError> {
    Ok(request.send().await?.error_for_status()?)
}

async fn receive<T: DeserializeOwned>(request: RequestBuilder) -> Result<T, ClientError> {
    Ok(send(request).await?.json().await?)
}

async fn read_capped(mut response: Response, limit: usize) -> Result<Vec<u8>, ClientError> {
    let mut body = Vec::new();

    while let Some(chunk) = response.chunk().await? {
        body.extend_from_slice(&chunk);

        if body.len() >= limit {
            body.truncate(limit);
            break;
        }
    }

    Ok(body)
}

#[async_trait]
impl ProsaApi for Client {
    async fn health(&self) -> Result<ProsaHealth, ClientError> {
        receive(self.http.get(format!("{}/health", self.url))).await
    }

    async fn jwks(&self) -> Result<JwkSet, ClientError> {
        receive(self.http.get(format!("{}/.well-known/jwks.json", self.url))).await
    }

    async fn identity(&self, api_key: &str) -> Result<ProsaIdentity, ClientError> {
        receive(self.request(Method::GET, "/auth/me", api_key)).await
    }

    async fn sync_device(&self, sync_token: Option<i64>, api_key: &str) -> Result<ProsaSync, ClientError> {
        let mut request = self.request(Method::GET, "/sync", api_key);

        if let Some(sync_token) = sync_token {
            request = request.query(&[("sync_token", sync_token.to_string())]);
        }

        receive(request).await
    }

    async fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, ClientError> {
        receive(self.request(Method::GET, &format!("/books/{book_id}/metadata"), api_key)).await
    }

    async fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, ClientError> {
        receive(self.request(Method::GET, &format!("/books/{book_id}/file-metadata"), api_key)).await
    }

    async fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        let response = send(self.request(Method::GET, &format!("/books/{book_id}"), api_key)).await?;

        read_capped(response, MAX_BOOK_SIZE).await
    }

    async fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), ClientError> {
        send(self.request(Method::DELETE, &format!("/books/{book_id}"), api_key)).await?;

        Ok(())
    }

    async fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        let response = send(self.request(Method::GET, &format!("/books/{book_id}/cover"), api_key)).await?;

        read_capped(response, MAX_COVER_SIZE).await
    }

    async fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, ClientError> {
        receive(self.request(Method::GET, &format!("/books/{book_id}/state"), api_key)).await
    }

    async fn patch_state(
        &self,
        book_id: &str,
        location: Option<&str>,
        reading_status: ProsaReadingStatus,
        api_key: &str,
    ) -> Result<(), ClientError> {
        let patch = ProsaStatePatch {
            location,
            statistics: ProsaStatisticsPatch {
                rating: None,
                reading_status: Some(reading_status),
            },
        };

        send(
            self.request(Method::PATCH, &format!("/books/{book_id}/state"), api_key)
                .json(&patch),
        )
        .await?;

        Ok(())
    }

    async fn replace_state(
        &self,
        book_id: &str,
        state: &ProsaState,
        api_key: &str,
    ) -> Result<(), ClientError> {
        send(
            self.request(Method::PUT, &format!("/books/{book_id}/state"), api_key)
                .json(state),
        )
        .await?;

        Ok(())
    }

    async fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        receive(self.request(Method::GET, &format!("/books/{book_id}/annotations"), api_key)).await
    }

    async fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, ClientError> {
        let path = format!("/books/{book_id}/annotations/{annotation_id}");

        receive(self.request(Method::GET, &path, api_key)).await
    }

    async fn add_annotation(
        &self,
        book_id: &str,
        annotation: &ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, ClientError> {
        let request = self
            .request(Method::POST, &format!("/books/{book_id}/annotations"), api_key)
            .json(annotation);

        Ok(send(request).await?.text().await?)
    }

    async fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        let path = format!("/books/{book_id}/annotations/{annotation_id}");

        send(
            self.request(Method::PATCH, &path, api_key)
                .json(&ProsaAnnotationPatch { note }),
        )
        .await?;

        Ok(())
    }

    async fn delete_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        let path = format!("/books/{book_id}/annotations/{annotation_id}");

        send(self.request(Method::DELETE, &path, api_key)).await?;

        Ok(())
    }

    async fn create_shelf(
        &self,
        shelf_name: &str,
        owner_id: Option<&str>,
        shelf_id: Option<&str>,
        api_key: &str,
    ) -> Result<String, ClientError> {
        let request = ProsaShelfCreateRequest {
            name: shelf_name,
            owner_id,
            shelf_id,
        };

        Ok(
            send(self.request(Method::POST, "/shelves", api_key).json(&request))
                .await?
                .text()
                .await?,
        )
    }

    async fn search_shelves(
        &self,
        username: &str,
        name: &str,
        api_key: &str,
    ) -> Result<Vec<String>, ClientError> {
        let mut shelf_ids = Vec::new();
        let mut page: u64 = 1;

        loop {
            let request = self.request(Method::GET, "/shelves", api_key).query(&[
                ("username", username),
                ("name", name),
                ("page", &page.to_string()),
                ("size", &SEARCH_PAGE_SIZE.to_string()),
            ]);
            let result: ProsaShelfSearch = receive(request).await?;
            shelf_ids.extend(result.shelf_ids);

            if result.current_page >= result.total_pages {
                return Ok(shelf_ids);
            }

            page += 1;
        }
    }

    async fn get_shelf_metadata(
        &self,
        shelf_id: &str,
        api_key: &str,
    ) -> Result<ProsaShelfMetadata, ClientError> {
        receive(self.request(Method::GET, &format!("/shelves/{shelf_id}"), api_key)).await
    }

    async fn update_shelf_name(
        &self,
        shelf_id: &str,
        shelf_name: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        let update = ProsaShelfUpdateRequest { name: shelf_name };

        send(
            self.request(Method::PUT, &format!("/shelves/{shelf_id}"), api_key)
                .json(&update),
        )
        .await?;

        Ok(())
    }

    async fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), ClientError> {
        send(self.request(Method::DELETE, &format!("/shelves/{shelf_id}"), api_key)).await?;

        Ok(())
    }

    async fn add_book_to_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        let book = ProsaAddBookShelfRequest { book_id };

        send(
            self.request(Method::POST, &format!("/shelves/{shelf_id}/books"), api_key)
                .json(&book),
        )
        .await?;

        Ok(())
    }

    async fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        receive(self.request(Method::GET, &format!("/shelves/{shelf_id}/books"), api_key)).await
    }

    async fn delete_book_from_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        let path = format!("/shelves/{shelf_id}/books/{book_id}");

        send(self.request(Method::DELETE, &path, api_key)).await?;

        Ok(())
    }
}

impl From<Error> for ClientError {
    fn from(value: Error) -> Self {
        value.status().map_or(ClientError::InternalError, |status| {
            ClientError::new(status.as_u16())
        })
    }
}

impl ClientError {
    pub fn new(status_code: u16) -> Self {
        match status_code {
            400 => ClientError::BadRequest,
            401 => ClientError::Unauthorized,
            403 => ClientError::Forbidden,
            404 => ClientError::NotFound,
            409 => ClientError::Conflict,
            _ => ClientError::InternalError,
        }
    }
}
