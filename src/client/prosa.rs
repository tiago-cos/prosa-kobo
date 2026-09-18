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
use crate::app::AppState;
use axum::extract::FromRef;
use jsonwebtoken::jwk::JwkSet;
use std::sync::Arc;
use strum_macros::{EnumMessage, EnumProperty};
use ureq::{Agent, Error};

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

/// The slice of the Prosa API the middleware speaks, kept as a trait so tests
/// can stand a fake in front of the services instead of a live backend.
pub trait ProsaApi: Send + Sync {
    fn health(&self) -> Result<ProsaHealth, ClientError>;

    fn jwks(&self) -> Result<JwkSet, ClientError>;

    fn identity(&self, api_key: &str) -> Result<ProsaIdentity, ClientError>;

    fn sync_device(&self, sync_token: Option<i64>, api_key: &str) -> Result<ProsaSync, ClientError>;

    fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, ClientError>;

    fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, ClientError>;

    fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError>;

    fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), ClientError>;

    fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError>;

    fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, ClientError>;

    fn patch_state(
        &self,
        book_id: &str,
        location: Option<&str>,
        reading_status: ProsaReadingStatus,
        api_key: &str,
    ) -> Result<(), ClientError>;

    fn update_rating(&self, book_id: &str, rating: u8, api_key: &str) -> Result<(), ClientError>;

    fn fetch_rating(&self, book_id: &str, api_key: &str) -> Result<Option<u8>, ClientError>;

    fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, ClientError>;

    fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, ClientError>;

    fn add_annotation(
        &self,
        book_id: &str,
        annotation: &ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, ClientError>;

    fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), ClientError>;

    fn delete_annotation(&self, book_id: &str, annotation_id: &str, api_key: &str)
    -> Result<(), ClientError>;

    fn create_shelf(
        &self,
        shelf_name: &str,
        owner_id: Option<&str>,
        shelf_id: Option<&str>,
        api_key: &str,
    ) -> Result<String, ClientError>;

    fn get_shelf_metadata(&self, shelf_id: &str, api_key: &str) -> Result<ProsaShelfMetadata, ClientError>;

    fn update_shelf_name(&self, shelf_id: &str, shelf_name: &str, api_key: &str) -> Result<(), ClientError>;

    fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), ClientError>;

    fn add_book_to_shelf(&self, shelf_id: &str, book_id: &str, api_key: &str) -> Result<(), ClientError>;

    fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, ClientError>;

    fn delete_book_from_shelf(&self, shelf_id: &str, book_id: &str, api_key: &str)
    -> Result<(), ClientError>;
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
        let agent: Agent = Agent::config_builder().build().into();
        let url = format!("{scheme}://{url}:{port}");

        Client {
            health_client: HealthClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            identity_client: IdentityClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            keys_client: KeysClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            sync_client: SyncClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            metadata_client: MetadataClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            state_client: StateClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            book_client: BookClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            cover_client: CoverClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            annotations_client: AnnotationsClient {
                url: url.clone(),
                agent: agent.clone(),
            },
            shelf_client: ShelfClient {
                url: url.clone(),
                agent: agent.clone(),
            },
        }
    }
}

impl ProsaApi for Client {
    fn health(&self) -> Result<ProsaHealth, ClientError> {
        Ok(self.health_client.health()?)
    }

    fn jwks(&self) -> Result<JwkSet, ClientError> {
        Ok(self.keys_client.jwks()?)
    }

    fn identity(&self, api_key: &str) -> Result<ProsaIdentity, ClientError> {
        Ok(self.identity_client.identity(api_key)?)
    }

    fn sync_device(&self, sync_token: Option<i64>, api_key: &str) -> Result<ProsaSync, ClientError> {
        Ok(self.sync_client.sync_device(sync_token, api_key)?)
    }

    fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, ClientError> {
        Ok(self.metadata_client.fetch_metadata(book_id, api_key)?)
    }

    fn fetch_book_file_metadata(
        &self,
        book_id: &str,
        api_key: &str,
    ) -> Result<ProsaBookFileMetadata, ClientError> {
        Ok(self.book_client.fetch_book_file_metadata(book_id, api_key)?)
    }

    fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        Ok(self.book_client.download_book(book_id, api_key)?)
    }

    fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), ClientError> {
        Ok(self.book_client.delete_book(book_id, api_key)?)
    }

    fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, ClientError> {
        Ok(self.cover_client.download_cover(book_id, api_key)?)
    }

    fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, ClientError> {
        Ok(self.state_client.fetch_state(book_id, api_key)?)
    }

    fn patch_state(
        &self,
        book_id: &str,
        location: Option<&str>,
        reading_status: ProsaReadingStatus,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.state_client
            .patch_state(book_id, location, reading_status, api_key)?;

        Ok(())
    }

    fn update_rating(&self, book_id: &str, rating: u8, api_key: &str) -> Result<(), ClientError> {
        let mut state = self.state_client.fetch_state(book_id, api_key)?;

        state.statistics.rating = match rating {
            0 => None,
            rating => Some(rating.into()),
        };

        self.state_client.replace_state(book_id, &state, api_key)?;

        Ok(())
    }

    fn fetch_rating(&self, book_id: &str, api_key: &str) -> Result<Option<u8>, ClientError> {
        let state = self.state_client.fetch_state(book_id, api_key)?;

        Ok(state.statistics.rating.map(round_rating))
    }

    fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        Ok(self.annotations_client.list_annotations(book_id, api_key)?)
    }

    fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, ClientError> {
        Ok(self
            .annotations_client
            .get_annotation(book_id, annotation_id, api_key)?)
    }

    fn add_annotation(
        &self,
        book_id: &str,
        annotation: &ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, ClientError> {
        Ok(self
            .annotations_client
            .add_annotation(book_id, annotation, api_key)?)
    }

    fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.annotations_client
            .patch_annotation(book_id, annotation_id, note, api_key)?;

        Ok(())
    }

    fn delete_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.annotations_client
            .delete_annotation(book_id, annotation_id, api_key)?;

        Ok(())
    }

    fn create_shelf(
        &self,
        shelf_name: &str,
        owner_id: Option<&str>,
        shelf_id: Option<&str>,
        api_key: &str,
    ) -> Result<String, ClientError> {
        Ok(self
            .shelf_client
            .create_shelf(shelf_name, owner_id, shelf_id, api_key)?)
    }

    fn get_shelf_metadata(&self, shelf_id: &str, api_key: &str) -> Result<ProsaShelfMetadata, ClientError> {
        Ok(self.shelf_client.get_shelf_metadata(shelf_id, api_key)?)
    }

    fn update_shelf_name(&self, shelf_id: &str, shelf_name: &str, api_key: &str) -> Result<(), ClientError> {
        self.shelf_client
            .update_shelf_name(shelf_id, shelf_name, api_key)?;

        Ok(())
    }

    fn delete_shelf(&self, shelf_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.shelf_client.delete_shelf(shelf_id, api_key)?;

        Ok(())
    }

    fn add_book_to_shelf(&self, shelf_id: &str, book_id: &str, api_key: &str) -> Result<(), ClientError> {
        self.shelf_client.add_book_to_shelf(shelf_id, book_id, api_key)?;

        Ok(())
    }

    fn list_books_in_shelf(&self, shelf_id: &str, api_key: &str) -> Result<Vec<String>, ClientError> {
        Ok(self.shelf_client.list_books_in_shelf(shelf_id, api_key)?)
    }

    fn delete_book_from_shelf(
        &self,
        shelf_id: &str,
        book_id: &str,
        api_key: &str,
    ) -> Result<(), ClientError> {
        self.shelf_client
            .delete_book_from_shelf(shelf_id, book_id, api_key)?;

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

    // Bounded above, so the cast cannot truncate.
    rating as u8
}

impl FromRef<AppState> for Arc<dyn ProsaApi> {
    fn from_ref(state: &AppState) -> Arc<dyn ProsaApi> {
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
