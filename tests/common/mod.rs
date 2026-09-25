#![allow(dead_code)]

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response},
};
use jsonwebtoken::jwk::JwkSet;
use prosa_kobo::{
    app::{
        AppState, ProsaClient, authentication::service::load_prosa_keys, devices, kepub::KepubCache, router,
    },
    client::{
        identity::{ProsaAuthType, ProsaIdentity},
        mock::MockProsaClient,
    },
    database,
};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};
use tower::ServiceExt;

const CACHE_SIZE: u64 = 16 * 1024 * 1024;

pub const API_KEY: &str = "anapikey";
pub const USER: &str = "user";
pub const DEVICE_HARDWARE_ID: &str = "N123456789012";
pub const HOST: &str = "middleware.test:5001";

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

    pub async fn unlink(&self, device_id: &str) {
        devices::service::unlink_device(&self.state.pool, device_id)
            .await
            .expect("Failed to unlink the device");
    }

    pub async fn link(&self, name: &str, api_key: &str) -> Device {
        self.client.seed_identity(
            api_key,
            ProsaIdentity {
                auth_type: ProsaAuthType::ApiKey,
                user_id: USER.to_owned(),
                is_admin: false,
                capabilities: vec!["Read".to_owned(), "Create".to_owned(), "Update".to_owned()],
                key_id: Some("key".to_owned()),
            },
        );

        let (device_id, lookup_key) = devices::service::link_device(
            &self.state.pool,
            self.state.prosa_client.as_ref(),
            USER,
            name,
            api_key,
        )
        .await
        .expect("Failed to link the device");

        Device {
            device_id,
            lookup_key,
        }
    }

    pub async fn linked(&self) -> Device {
        self.link("Kobo", API_KEY).await
    }

    pub fn app(&self) -> Router {
        router(&self.state)
    }

    pub async fn send(&self, request: Request<Body>) -> Response<Body> {
        self.app().oneshot(request).await.expect("The router failed")
    }

    pub async fn get(&self, uri: &str) -> Response<Body> {
        self.request(Method::GET, uri, &[]).await
    }

    pub async fn request(&self, method: Method, uri: &str, headers: &[(&str, &str)]) -> Response<Body> {
        let mut request = Request::builder().method(method).uri(uri).header("Host", HOST);

        for (name, value) in headers {
            request = request.header(*name, *value);
        }

        let request = request.body(Body::empty()).expect("Failed to build the request");

        self.send(request).await
    }

    pub async fn json(&self, method: Method, uri: &str, body: Value) -> Response<Body> {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("Host", HOST)
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .expect("Failed to build the request");

        self.send(request).await
    }
}

pub struct Device {
    pub device_id: String,
    pub lookup_key: String,
}

impl Device {
    pub fn at(&self, path: &str) -> String {
        format!("/{}{path}", self.lookup_key)
    }
}

pub async fn body_json(response: Response<Body>) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read the body");

    serde_json::from_slice(&bytes).expect("The body was not JSON")
}

pub async fn body_bytes(response: Response<Body>) -> Vec<u8> {
    to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read the body")
        .to_vec()
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.database);
    }
}
