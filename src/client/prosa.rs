use super::{
    annotations::{AnnotationsClient, ProsaAnnotation},
    book::BookClient,
    cover::CoverClient,
    metadata::{MetadataClient, ProsaMetadata},
    state::StateClient,
    sync::{ProsaSync, SyncClient},
    ProsaAnnotationRequest, ProsaState,
};
use crate::app::AppState;
use axum::extract::FromRef;
use std::sync::Arc;
use ureq::{Agent, Error};

pub struct Client {
    url: String,
    agent: Agent,
    sync_client: SyncClient,
    metadata_client: MetadataClient,
    state_client: StateClient,
    book_client: BookClient,
    cover_client: CoverClient,
    annotations_client: AnnotationsClient,
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
        }
    }

    pub fn sync_device(&self, since: Option<i64>, api_key: &str) -> Result<ProsaSync, Error> {
        self.sync_client
            .sync_device(&self.url, &self.agent, since, api_key)
    }

    pub fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<ProsaMetadata, Error> {
        self.metadata_client
            .fetch_metadata(&self.url, &self.agent, book_id, api_key)
    }

    pub fn fetch_size(&self, book_id: &str, api_key: &str) -> Result<u64, Error> {
        self.metadata_client
            .fetch_size(&self.url, &self.agent, book_id, api_key)
    }

    pub fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<ProsaState, Error> {
        self.state_client
            .fetch_state(&self.url, &self.agent, book_id, api_key)
    }

    pub fn update_state(
        &self,
        book_id: &str,
        tag: Option<String>,
        source: Option<String>,
        reading_status: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        self.state_client.update_state(
            &self.url,
            &self.agent,
            book_id,
            tag,
            source,
            reading_status,
            api_key,
        )
    }

    pub fn update_rating(&self, book_id: &str, rating: u8, api_key: &str) -> Result<(), Error> {
        self.state_client
            .update_rating(&self.url, &self.agent, book_id, rating, api_key)
    }

    pub fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        self.book_client
            .download_book(&self.url, &self.agent, book_id, api_key)
    }

    pub fn delete_book(&self, book_id: &str, api_key: &str) -> Result<(), Error> {
        self.book_client
            .delete_book(&self.url, &self.agent, book_id, api_key)
    }

    pub fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        self.cover_client
            .download_cover(&self.url, &self.agent, book_id, api_key)
    }

    pub fn list_annotations(&self, book_id: &str, api_key: &str) -> Result<Vec<String>, Error> {
        self.annotations_client
            .list_annotations(&self.url, &self.agent, book_id, api_key)
    }

    pub fn get_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        api_key: &str,
    ) -> Result<ProsaAnnotation, Error> {
        self.annotations_client
            .get_annotation(&self.url, &self.agent, book_id, annotation_id, api_key)
    }

    pub fn add_annotation(
        &self,
        book_id: &str,
        annotation: ProsaAnnotationRequest,
        api_key: &str,
    ) -> Result<String, Error> {
        self.annotations_client
            .add_annotation(&self.url, &self.agent, book_id, annotation, api_key)
    }

    pub fn patch_annotation(
        &self,
        book_id: &str,
        annotation_id: &str,
        note: &str,
        api_key: &str,
    ) -> Result<(), Error> {
        self.annotations_client.patch_annotation(
            &self.url,
            &self.agent,
            book_id,
            annotation_id,
            note,
            api_key,
        )
    }

    pub fn delete_annotation(&self, book_id: &str, annotation_id: &str, api_key: &str) -> Result<(), Error> {
        self.annotations_client
            .delete_annotation(&self.url, &self.agent, book_id, annotation_id, api_key)
    }
}

impl FromRef<AppState> for Arc<Client> {
    fn from_ref(state: &AppState) -> Arc<Client> {
        Arc::clone(&state.prosa_client)
    }
}
