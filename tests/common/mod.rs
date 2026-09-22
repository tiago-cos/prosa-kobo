#![allow(dead_code)]

use axum::{
    Router,
    body::Body,
    http::{Request, Response},
};
use jsonwebtoken::jwk::JwkSet;
use prosa_kobo::{
    app::{AppState, ProsaClient, authentication::service::load_prosa_keys, kepub::KepubCache, router},
    client::mock::MockProsaClient,
    database,
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};
use tower::ServiceExt;

const CACHE_SIZE: u64 = 16 * 1024 * 1024;

/// A middleware wired to an in-memory Prosa and a database of its own, so a
/// test can drive the real router without either running anywhere.
pub struct Harness {
    pub client: Arc<MockProsaClient>,
    pub state: AppState,
    database: PathBuf,
}

impl Harness {
    pub async fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);

        let database = std::env::temp_dir().join(format!(
            "prosa-kobo-{}-{}.db",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));

        let client = Arc::new(MockProsaClient::new());

        // A suite that signs its own tokens seeds the matching key set instead.
        client.seed_jwks(JwkSet { keys: Vec::new() });
        load_prosa_keys(client.as_ref());

        let state = AppState {
            prosa_client: Arc::clone(&client) as ProsaClient,
            pool: Arc::new(database::init(&database.to_string_lossy()).await),
            kepubs: Arc::new(KepubCache::new(CACHE_SIZE)),
        };

        Self {
            client,
            state,
            database,
        }
    }

    pub fn app(&self) -> Router {
        router(&self.state)
    }

    pub async fn send(&self, request: Request<Body>) -> Response<Body> {
        self.app().oneshot(request).await.expect("The router failed")
    }

    pub async fn get(&self, uri: &str) -> Response<Body> {
        let request = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .expect("Failed to build the request");

        self.send(request).await
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.database);
    }
}
