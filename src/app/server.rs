use crate::config::Configuration;
use axum::Router;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::net::TcpListener;

use super::devices;

pub type Config = Arc<Configuration>;
pub type Pool = Arc<SqlitePool>;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Pool,
}

pub async fn run(config: Configuration, pool: SqlitePool) {
    let state = AppState {
        config: Arc::new(config),
        pool: Arc::new(pool),
    };
    let host = format!("{}:{}", &state.config.server.host, &state.config.server.port);
    let app = Router::new().merge(devices::routes::get_routes(state.clone()));

    let listener = TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
