pub mod annotations;
pub mod book;
pub mod cover;
pub mod health;
pub mod identity;
pub mod keys;
pub mod metadata;
pub mod prosa;
pub mod shelf;
pub mod state;
pub mod sync;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

pub use annotations::{ProsaAnnotation, ProsaAnnotationRequest};
pub use metadata::ProsaMetadata;
pub use state::ProsaReadingStatus;

use prosa::ProsaApi;
use std::sync::{Arc, OnceLock};

static PROSA_CLIENT: OnceLock<Arc<dyn ProsaApi>> = OnceLock::new();

pub fn set_prosa_client(client: Arc<dyn ProsaApi>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    PROSA_CLIENT
        .set(client)
        .map_err(|_| "Prosa client was already initialized".into())
}

pub fn prosa_client() -> &'static dyn ProsaApi {
    PROSA_CLIENT
        .get()
        .expect("Prosa client accessed before initialization")
        .as_ref()
}
