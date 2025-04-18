use super::{
    book::BookClient,
    cover::CoverClient,
    metadata::{MetadataClient, MetadataResponse},
    state::StateClient,
    sync::{SyncClient, SyncResponse},
    StateResponse,
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
        }
    }

    pub fn sync_device(&self, since: Option<i64>, api_key: &str) -> Result<SyncResponse, Error> {
        self.sync_client
            .sync_device(&self.url, &self.agent, since, api_key)
    }

    pub fn fetch_metadata(&self, book_id: &str, api_key: &str) -> Result<MetadataResponse, Error> {
        self.metadata_client
            .fetch_metadata(&self.url, &self.agent, book_id, api_key)
    }

    pub fn fetch_size(&self, book_id: &str, api_key: &str) -> Result<u64, Error> {
        self.metadata_client
            .fetch_size(&self.url, &self.agent, book_id, api_key)
    }

    pub fn fetch_state(&self, book_id: &str, api_key: &str) -> Result<StateResponse, Error> {
        self.state_client
            .fetch_state(&self.url, &self.agent, book_id, api_key)
    }

    pub fn download_book(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        self.book_client
            .download_book(&self.url, &self.agent, book_id, api_key)
    }

    pub fn download_cover(&self, book_id: &str, api_key: &str) -> Result<Vec<u8>, Error> {
        self.cover_client
            .download_cover(&self.url, &self.agent, book_id, api_key)
    }
}

impl FromRef<AppState> for Arc<Client> {
    fn from_ref(state: &AppState) -> Arc<Client> {
        Arc::clone(&state.prosa_client)
    }
}
