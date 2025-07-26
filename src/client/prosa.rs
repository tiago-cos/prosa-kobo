use super::{
    annotations::{AnnotationsClient, ProsaAnnotation},
    book::BookClient,
    cover::CoverClient,
    metadata::{MetadataClient, ProsaMetadata},
    state::StateClient,
    sync::{ProsaSync, SyncClient},
    ProsaAnnotationRequest, ProsaState,
};
use crate::{
    app::AppState,
    client::{
        book::ProsaBookFileMetadata,
        shelf::{ProsaShelfMetadata, ShelfClient},
    },
};
use axum::extract::FromRef;
use std::sync::Arc;
use strum_macros::{EnumMessage, EnumProperty};
use ureq::{Agent, Error};

#[derive(EnumMessage, EnumProperty, Debug)]
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
pub struct Client {
    url: String,
    agent: Agent,
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
        let agent: Agent = Agent::config_builder().build().into();

        Client {
            url: format!("{}://{}:{}", scheme, url, port),
            agent,
            sync_client: SyncClient {},
            metadata_client: MetadataClient {},
            state_client: StateClient {},
            book_client: BookClient {},
            cover_client: CoverClient {},
            annotations_client: AnnotationsClient {},
            shelf_client: ShelfClient {},
        }
    }

    pub fn sync_device(&self, since: Option<i64>, api_key: &str) -> Result<ProsaSync, ClientError> {
        let result = self
            .sync_client
            .sync_device(&self.url, &self.agent, since, api_key)?;
        Ok(result)
    }

    pub fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, ClientError> {
        let result = self
            .metadata_client
            .fetch_metadata(&self.url, &self.agent, book_id, api_key)?;
        Ok(result)
    }

    pub fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, ClientError> {
        let result = self
            .book_client
            .fetch_book_file_metadata(&self.url, &self.agent, book_id, api_key)?;
        Ok(result)
    }

    pub fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, ClientError> {
        let result = self
            .state_client
            .fetch_state(&self.url, &self.agent, book_id, api_key)?;
        Ok(result)
    }

    pub fn patch_state(
        &self,
        book_id: &str,
        tag: Option<String>,
        source: Option<String>,
        reading_status: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.state_client.patch_state(
            &self.url,
            &self.agent,
            book_id,
            tag,
            source,
            reading_status,
            api_key,
        )?;
        Ok(())
    }

    pub fn update_rating(&self, book_id: &str, rating: u8, api_key: &str) -> Result<(), ClientError> {
        self.state_client
            .update_rating(&self.url, &self.agent, book_id, rating, api_key)?;
        Ok(())
    }

    pub fn fetch_rating(&self, book_id: &str, api_key: &str) -> Result<Option<u8>, ClientError> {
        let result = self
            .state_client
            .fetch_rating(&self.url, &self.agent, book_id, api_key)?;
        Ok(result)
    }

    pub fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        let result = self
            .book_client
            .download_book(&self.url, &self.agent, book_id, api_key)?;
        Ok(result)
    }

    pub fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.book_client
            .delete_book(&self.url, &self.agent, book_id, api_key)?;
        Ok(())
    }

    pub fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        let result = self
            .cover_client
            .download_cover(&self.url, &self.agent, book_id, api_key)?;
        Ok(result)
    }

    pub fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        let result = self
            .annotations_client
            .list_annotations(&self.url, &self.agent, book_id, api_key)?;
        Ok(result)
    }

    pub fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, ClientError> {
        let result = self.annotations_client.get_annotation(
            &self.url,
            &self.agent,
            book_id,
            annotation_id,
            api_key,
        )?;
        Ok(result)
    }

    pub fn add_annotation(
        &self,
        book_id: &str,
        annotation: ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, ClientError> {
        let result =
            self.annotations_client
                .add_annotation(&self.url, &self.agent, book_id, annotation, api_key)?;
        Ok(result)
    }

    pub fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        let result = self.annotations_client.patch_annotation(
            &self.url,
            &self.agent,
            book_id,
            annotation_id,
            note,
            api_key,
        )?;
        Ok(result)
    }

    pub fn delete_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.annotations_client
            .delete_annotation(&self.url, &self.agent, book_id, annotation_id, api_key)?;
        Ok(())
    }

    pub fn create_shelf(
        &self,
        shelf_name: &str,
        owner_id: Option<String>,
        api_key: &str,
    ) -> Result<String, ClientError> {
        let result = self
            .shelf_client
            .create_shelf(&self.url, &self.agent, shelf_name, owner_id, api_key)?;
        Ok(result)
    }

    pub fn get_shelf_metadata(
        &self,
        shelf_id: &str,
        api_key: &str,
    ) -> Result<ProsaShelfMetadata, ClientError> {
        let result = self
            .shelf_client
            .get_shelf_metadata(&self.url, &self.agent, shelf_id, api_key)?;
        Ok(result)
    }

    pub fn update_shelf_name(
        &self,
        shelf_id: &str,
        shelf_name: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.shelf_client
            .update_shelf_name(&self.url, &self.agent, shelf_id, shelf_name, api_key)?;
        Ok(())
    }

    pub fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.shelf_client
            .delete_shelf(&self.url, &self.agent, shelf_id, api_key)?;
        Ok(())
    }

    pub fn add_book_to_shelf(&self, shelf_id: &str, book_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.shelf_client
            .add_book_to_shelf(&self.url, &self.agent, shelf_id, book_id, api_key)?;
        Ok(())
    }

    pub fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        let result = self
            .shelf_client
            .list_books_in_shelf(&self.url, &self.agent, shelf_id, api_key)?;
        Ok(result)
    }

    pub fn delete_book_from_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.shelf_client
            .delete_book_from_shelf(&self.url, &self.agent, shelf_id, book_id, api_key)?;
        Ok(())
    }
}

impl FromRef<AppState> for Arc<Client> {
    fn from_ref(state: &AppState) -> Arc<Client> {
        Arc::clone(&state.prosa_client)
    }
}

impl From<Error> for ClientError {
    fn from(value: Error) -> Self {
        match value {
            Error::StatusCode(code) => ClientError::new(code),
            _ => ClientError::InternalError,
        }
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
