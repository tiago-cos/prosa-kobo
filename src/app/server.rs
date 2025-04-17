use super::{authentication, devices, initialization, proxy, sync};
use crate::{client::prosa::Client, config::Configuration};
use axum::Router;
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
    let host = format!("{}:{}", &state.config.server.host, &state.config.server.port);
    let app = Router::new()
        .merge(devices::routes::get_routes(state.clone()))
        .merge(initialization::routes::get_routes(state.clone()))
        .merge(sync::routes::get_routes(state.clone()))
        .merge(authentication::routes::get_routes(state.clone()))
        .merge(proxy::routes::get_routes(state.clone()));

    let listener = TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
