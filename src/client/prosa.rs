use super::{
    annotations::{AnnotationsClient, ProsaAnnotation, ProsaAnnotationRequest},
    book::{BookClient, ProsaBookFileMetadata},
    cover::CoverClient,
    health::{HealthClient, ProsaHealth},
    identity::{IdentityClient, ProsaIdentity},
    keys::KeysClient,
    metadata::{MetadataClient, ProsaMetadata},
    shelf::{ProsaShelfMetadata, ShelfClient},
    state::{ProsaReadingStatus, ProsaState, StateClient},
    sync::{ProsaSync, SyncClient},
};
use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;
use reqwest::Error;
use strum_macros::{EnumMessage, EnumProperty};

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

    async fn update_rating(&self, book_id: &str, rating: u8, api_key: &str) -> Result<(), ClientError>;

    async fn fetch_rating(&self, book_id: &str, api_key: &str) -> Result<Option<u8>, ClientError>;

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
    health_client: HealthClient,
    identity_client: IdentityClient,
    keys_client: KeysClient,
    sync_client: SyncClient,
    metadata_client: MetadataClient,
    state_client: StateClient,
    book_client: BookClient,
    cover_client: CoverClient,
    annotations_client: AnnotationsClient,
    shelf_client: ShelfClient,
}

impl Client {
    pub fn new(scheme: &str, url: &str, port: u16) -> Self {
        let http = reqwest::Client::new();
        let url = format!("{scheme}://{url}:{port}");

        Client {
            health_client: HealthClient {
                url: url.clone(),
                http: http.clone(),
            },
            identity_client: IdentityClient {
                url: url.clone(),
                http: http.clone(),
            },
            keys_client: KeysClient {
                url: url.clone(),
                http: http.clone(),
            },
            sync_client: SyncClient {
                url: url.clone(),
                http: http.clone(),
            },
            metadata_client: MetadataClient {
                url: url.clone(),
                http: http.clone(),
            },
            state_client: StateClient {
                url: url.clone(),
                http: http.clone(),
            },
            book_client: BookClient {
                url: url.clone(),
                http: http.clone(),
            },
            cover_client: CoverClient {
                url: url.clone(),
                http: http.clone(),
            },
            annotations_client: AnnotationsClient {
                url: url.clone(),
                http: http.clone(),
            },
            shelf_client: ShelfClient {
                url: url.clone(),
                http: http.clone(),
            },
        }
    }
}

#[async_trait]
impl ProsaApi for Client {
    async fn health(&self) -> Result<ProsaHealth, ClientError> {
        Ok(self.health_client.health().await?)
    }

    async fn jwks(&self) -> Result<JwkSet, ClientError> {
        Ok(self.keys_client.jwks().await?)
    }

    async fn identity(&self, api_key: &str) -> Result<ProsaIdentity, ClientError> {
        Ok(self.identity_client.identity(api_key).await?)
    }

    async fn sync_device(&self, sync_token: Option<i64>, api_key: &str) -> Result<ProsaSync, ClientError> {
        Ok(self.sync_client.sync_device(sync_token, api_key).await?)
    }

    async fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, ClientError> {
        Ok(self.metadata_client.fetch_metadata(book_id, api_key).await?)
    }

    async fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, ClientError> {
        Ok(self
            .book_client
            .fetch_book_file_metadata(book_id, api_key)
            .await?)
    }

    async fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        Ok(self.book_client.download_book(book_id, api_key).await?)
    }

    async fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), ClientError> {
        Ok(self.book_client.delete_book(book_id, api_key).await?)
    }

    async fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        Ok(self.cover_client.download_cover(book_id, api_key).await?)
    }

    async fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, ClientError> {
        Ok(self.state_client.fetch_state(book_id, api_key).await?)
    }

    async fn patch_state(
        &self,
        book_id: &str,
        location: Option<&str>,
        reading_status: ProsaReadingStatus,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.state_client
            .patch_state(book_id, location, reading_status, api_key)
            .await?;

        Ok(())
    }

    async fn update_rating(&self, book_id: &str, rating: u8, api_key: &str) -> Result<(), ClientError> {
        let mut state = self.state_client.fetch_state(book_id, api_key).await?;

        state.statistics.rating = match rating {
            0 => None,
            rating => Some(rating.into()),
        };

        self.state_client.replace_state(book_id, &state, api_key).await?;

        Ok(())
    }

    async fn fetch_rating(&self, book_id: &str, api_key: &str) -> Result<Option<u8>, ClientError> {
        let state = self.state_client.fetch_state(book_id, api_key).await?;

        Ok(state.statistics.rating.map(round_rating))
    }

    async fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        Ok(self.annotations_client.list_annotations(book_id, api_key).await?)
    }

    async fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, ClientError> {
        Ok(self
            .annotations_client
            .get_annotation(book_id, annotation_id, api_key)
            .await?)
    }

    async fn add_annotation(
        &self,
        book_id: &str,
        annotation: &ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, ClientError> {
        Ok(self
            .annotations_client
            .add_annotation(book_id, annotation, api_key)
            .await?)
    }

    async fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.annotations_client
            .patch_annotation(book_id, annotation_id, note, api_key)
            .await?;

        Ok(())
    }

    async fn delete_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.annotations_client
            .delete_annotation(book_id, annotation_id, api_key)
            .await?;

        Ok(())
    }

    async fn create_shelf(
        &self,
        shelf_name: &str,
        owner_id: Option<&str>,
        shelf_id: Option<&str>,
        api_key: &str,
    ) -> Result<String, ClientError> {
        Ok(self
            .shelf_client
            .create_shelf(shelf_name, owner_id, shelf_id, api_key)
            .await?)
    }

    async fn search_shelves(
        &self,
        username: &str,
        name: &str,
        api_key: &str,
    ) -> Result<Vec<String>, ClientError> {
        let mut shelf_ids = Vec::new();
        let mut page = 1;

        loop {
            let result = self
                .shelf_client
                .search_shelves(username, name, page, api_key)
                .await?;
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
        Ok(self.shelf_client.get_shelf_metadata(shelf_id, api_key).await?)
    }

    async fn update_shelf_name(
        &self,
        shelf_id: &str,
        shelf_name: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.shelf_client
            .update_shelf_name(shelf_id, shelf_name, api_key)
            .await?;

        Ok(())
    }

    async fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.shelf_client.delete_shelf(shelf_id, api_key).await?;

        Ok(())
    }

    async fn add_book_to_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.shelf_client
            .add_book_to_shelf(shelf_id, book_id, api_key)
            .await?;

        Ok(())
    }

    async fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        Ok(self.shelf_client.list_books_in_shelf(shelf_id, api_key).await?)
    }

    async fn delete_book_from_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.shelf_client
            .delete_book_from_shelf(shelf_id, book_id, api_key)
            .await?;

        Ok(())
    }
}

fn round_rating(rating: f32) -> u8 {
    let rating = rating.round();

    if rating <= 0.0 {
        return 0;
    }

    if rating >= f32::from(u8::MAX) {
        return u8::MAX;
    }

    rating as u8
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
