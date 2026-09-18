use super::{
    annotations, authentication, books, covers, devices, initialization, metadata, proxy, state, sync,
};
use crate::{
    app::{shelves, tracing},
    client::prosa::{Client, ProsaApi},
    config::Configuration,
};
use axum::{Router, http::StatusCode, middleware::from_fn, routing::get};
use log::{error, info, warn};
use sqlx::SqlitePool;
use std::{process::exit, sync::Arc, time::Duration};
use tokio::{net::TcpListener, time::sleep};

const EXPECTED_PROSA_VERSION: &str = "0.2.0";

const PROSA_RETRY_INTERVAL: Duration = Duration::from_secs(5);

pub type Config = Arc<Configuration>;
pub type Pool = Arc<SqlitePool>;
pub type ProsaClient = Arc<dyn ProsaApi>;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Pool,
    pub prosa_client: ProsaClient,
}

pub async fn run(config: Configuration, pool: SqlitePool) {
    tracing::init_logging();

    let prosa_url = format!(
        "{}://{}:{}",
        config.prosa.scheme, config.prosa.host, config.prosa.port
    );

    let prosa_client = Arc::new(Client::new(
        &config.prosa.scheme,
        &config.prosa.host,
        config.prosa.port,
    ));

    await_prosa(prosa_client.as_ref(), &prosa_url).await;

    let state = AppState {
        prosa_client,
        config: Arc::new(config),
        pool: Arc::new(pool),
    };

    let host = format!(
        "{}:{}",
        &state.config.server.bind.host, &state.config.server.bind.port
    );

    info!("Middleware started on http://{host}");

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::NO_CONTENT }))
        .merge(devices::routes::get_routes(state.clone()))
        .merge(initialization::routes::get_routes(state.clone()))
        .merge(sync::routes::get_routes(state.clone()))
        .merge(authentication::routes::get_routes(state.clone()))
        .merge(metadata::routes::get_routes(state.clone()))
        .merge(books::routes::get_routes(state.clone()))
        .merge(covers::routes::get_routes(state.clone()))
        .merge(state::routes::get_routes(state.clone()))
        .merge(annotations::routes::get_routes(state.clone()))
        .merge(shelves::routes::get_routes(state.clone()))
        .merge(proxy::routes::get_routes(state.clone()))
        .layer(from_fn(tracing::log_layer));

    let listener = TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn await_prosa(client: &dyn ProsaApi, prosa_url: &str) {
    let health = loop {
        match client.health() {
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
