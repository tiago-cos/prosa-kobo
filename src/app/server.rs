use super::{
    annotations, authentication, books, covers, devices, fallback, initialization, kepub, metadata, state,
    sync,
};
use crate::{
    CONFIG,
    app::{authentication::middleware::extract_device_middleware, shelves, tracing},
    client::{prosa::ProsaApi, prosa_client},
};
use axum::{Router, http::StatusCode, middleware::from_fn, routing::get};
use log::{error, info, warn};
use std::{process::exit, sync::LazyLock, time::Duration};
use tokio::{net::TcpListener, time::sleep};
use tower::ServiceBuilder;

const EXPECTED_PROSA_VERSION: &str = "0.2.0";

const PROSA_RETRY_INTERVAL: Duration = Duration::from_secs(5);

pub static KEPUBS: LazyLock<kepub::KepubCache> =
    LazyLock::new(|| kepub::KepubCache::new(CONFIG.kepub.cache_size_mb * 1024 * 1024));

pub async fn run() {
    let prosa_url = format!(
        "{}://{}:{}",
        CONFIG.prosa.scheme, CONFIG.prosa.host, CONFIG.prosa.port
    );

    await_prosa(prosa_client(), &prosa_url).await;

    authentication::service::load_prosa_keys(prosa_client()).await;

    let host = format!("{}:{}", CONFIG.server.bind.host, CONFIG.server.bind.port);

    info!("Middleware started on http://{host}");

    let app = router();

    let listener = TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub fn router() -> Router {
    let device = Router::new()
        .merge(initialization::routes::get_routes())
        .merge(sync::routes::get_routes())
        .merge(authentication::routes::get_routes())
        .merge(metadata::routes::get_routes())
        .merge(books::routes::get_routes())
        .merge(covers::routes::get_routes())
        .merge(state::routes::get_routes())
        .merge(annotations::routes::get_routes())
        .merge(shelves::routes::get_routes())
        .merge(fallback::routes::get_routes());

    let device = ServiceBuilder::new()
        .layer(from_fn(extract_device_middleware))
        .service(device);

    Router::new()
        .route("/health", get(|| async { StatusCode::NO_CONTENT }))
        .merge(devices::routes::get_routes())
        .fallback_service(device)
        .layer(from_fn(tracing::log_layer))
}

async fn await_prosa(client: &dyn ProsaApi, prosa_url: &str) {
    let health = loop {
        match client.health().await {
            Ok(health) => break health,
            Err(error) => {
                warn!(
                    "Could not reach Prosa at {prosa_url} ({error:?}), retrying in {} seconds",
                    PROSA_RETRY_INTERVAL.as_secs()
                );

                sleep(PROSA_RETRY_INTERVAL).await;
            }
        }
    };

    if !is_compatible(&health.version) {
        error!(
            "Prosa at {prosa_url} reports version {}, but this middleware speaks {EXPECTED_PROSA_VERSION}",
            health.version
        );

        exit(1);
    }

    info!(
        "Connected to {} {} at {prosa_url}",
        health.software, health.version
    );
}

fn is_compatible(reported: &str) -> bool {
    fn major_minor(version: &str) -> Option<(&str, &str)> {
        let mut parts = version.split('.');
        Some((parts.next()?, parts.next()?))
    }

    match (major_minor(reported), major_minor(EXPECTED_PROSA_VERSION)) {
        (Some(reported), Some(expected)) => reported == expected,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_version_it_was_written_against() {
        assert!(is_compatible(EXPECTED_PROSA_VERSION));
    }

    #[test]
    fn accepts_a_patch_release() {
        assert!(is_compatible("0.2.7"));
    }

    #[test]
    fn rejects_a_different_minor_or_major() {
        assert!(!is_compatible("0.3.0"));
        assert!(!is_compatible("1.2.0"));
        assert!(!is_compatible("0.20.0"));
    }

    #[test]
    fn rejects_a_version_it_cannot_read() {
        assert!(!is_compatible(""));
        assert!(!is_compatible("0"));
        assert!(!is_compatible("unreleased"));
    }
}
