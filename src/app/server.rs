use super::{
    annotations, authentication, books, covers, devices, initialization, metadata, proxy, state, sync,
};
use crate::{
    app::{shelves, tracing},
    client::prosa::Client,
    config::Configuration,
};
use axum::{Router, http::StatusCode, middleware::from_fn, routing::get};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::net::TcpListener;

pub type Config = Arc<Configuration>;
pub type Pool = Arc<SqlitePool>;
pub type ProsaClient = Arc<Client>;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Pool,
    pub prosa_client: ProsaClient,
}

pub async fn run(config: Configuration, pool: SqlitePool) {
    let state = AppState {
        prosa_client: Arc::new(Client::new(
            &config.prosa.scheme,
            &config.prosa.host,
            config.prosa.port,
        )),
        config: Arc::new(config),
        pool: Arc::new(pool),
    };

    tracing::init_logging();

    let host = format!(
        "{}:{}",
        &state.config.server.bind.host, &state.config.server.bind.port
    );
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
