use super::{
    annotations, authentication, books, covers, devices, initialization, kepub, metadata, proxy, state, sync,
};
use crate::{
    CONFIG,
    app::{authentication::middleware::extract_device_middleware, shelves, tracing},
    client::{prosa::ProsaApi, prosa_client},
};
use axum::{
    Router,
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header, request::Parts},
    middleware::from_fn,
    routing::get,
};
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
        .merge(proxy::routes::get_routes());

    let device = ServiceBuilder::new()
        .layer(from_fn(extract_device_middleware))
        .service(device);

    Router::new()
        .route("/health", get(|| async { StatusCode::NO_CONTENT }))
        .merge(devices::routes::get_routes())
        .fallback_service(device)
        .layer(from_fn(tracing::log_layer))
}

pub struct Host(pub String);

impl<S: Send + Sync> FromRequestParts<S> for Host {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        forwarded_host(&parts.headers)
            .or_else(|| header_value(&parts.headers, "x-forwarded-host"))
            .or_else(|| header_value(&parts.headers, header::HOST.as_str()))
            .or_else(|| {
                parts
                    .uri
                    .authority()
                    .and_then(|authority| authority.as_str().rsplit('@').next())
            })
            .map(|host| Host(host.to_owned()))
            .ok_or((StatusCode::BAD_REQUEST, "No host found in request"))
    }
}

fn forwarded_host(headers: &HeaderMap) -> Option<&str> {
    header_value(headers, header::FORWARDED.as_str())?
        .split(',')
        .next()?
        .split(';')
        .find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case("host")
                .then(|| value.trim().trim_matches('"'))
        })
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

pub fn device_url(host: &str, lookup_key: &str) -> String {
    format!("{}/{lookup_key}", server_url(host))
}

fn server_url(host: &str) -> String {
    match &CONFIG.server.public {
        Some(public) => format!("{}://{}:{}", public.scheme, public.host, public.port),
        None if host.contains(':') => format!("http://{host}"),
        _ => format!("http://{host}:{}", CONFIG.server.bind.port),
    }
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

    async fn host(request: axum::http::request::Builder) -> Result<String, StatusCode> {
        let (mut parts, ()) = request.body(()).expect("Failed to build a request").into_parts();

        Host::from_request_parts(&mut parts, &())
            .await
            .map(|Host(host)| host)
            .map_err(|(status, _)| status)
    }

    #[tokio::test]
    async fn prefers_the_forwarded_host_over_every_other_source() {
        let request = axum::http::Request::builder()
            .uri("http://authority.test/")
            .header("host", "host.test")
            .header("x-forwarded-host", "x-forwarded.test")
            .header(
                "forwarded",
                r#"for=192.0.2.60;Host="forwarded.test:8443";proto=https, host=second.test"#,
            );

        assert_eq!(host(request).await, Ok("forwarded.test:8443".to_owned()));
    }

    #[tokio::test]
    async fn falls_back_to_x_forwarded_host_then_host() {
        let forwarded = axum::http::Request::builder()
            .header("host", "host.test")
            .header("x-forwarded-host", "x-forwarded.test");
        let plain = axum::http::Request::builder().header("host", "host.test:5001");

        assert_eq!(host(forwarded).await, Ok("x-forwarded.test".to_owned()));
        assert_eq!(host(plain).await, Ok("host.test:5001".to_owned()));
    }

    #[tokio::test]
    async fn takes_the_authority_without_its_credentials_when_no_header_names_a_host() {
        let request = axum::http::Request::builder().uri("http://user:secret@authority.test:5001/");

        assert_eq!(host(request).await, Ok("authority.test:5001".to_owned()));
    }

    #[tokio::test]
    async fn refuses_a_request_that_names_no_host() {
        let request = axum::http::Request::builder().uri("/v1/initialization");

        assert_eq!(host(request).await, Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn keeps_the_port_the_device_reached_it_on() {
        assert_eq!(
            device_url("middleware.test:8080", "key"),
            "http://middleware.test:8080/key"
        );
    }

    #[test]
    fn names_the_bound_port_when_the_host_carries_none() {
        assert_eq!(
            device_url("middleware.test", "key"),
            format!("http://middleware.test:{}/key", CONFIG.server.bind.port)
        );
    }
}
