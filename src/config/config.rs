use crate::app::AppState;
use axum::extract::FromRef;
use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub announced_host: String,
    pub announced_port: u16,
}

#[derive(Deserialize)]
pub struct Prosa {
    pub host: String,
    pub port: u16,
    pub scheme: String,
}

#[derive(Deserialize)]
pub struct DownloadToken {
    pub book_expiration: i64,
}

#[derive(Deserialize)]
pub struct Configuration {
    pub server: Server,
    pub database: Database,
    pub auth: Auth,
    pub prosa: Prosa,
    pub download_token: DownloadToken,
}

#[derive(Deserialize, Clone)]
pub struct Database {
    pub file_path: String,
}

#[derive(Deserialize, Clone)]
pub struct Auth {
    pub secret_key: String,
    pub token_duration: u64,
    pub refresh_token_duration: u64,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let conf = Config::builder()
            .add_source(File::with_name("src/config/default.toml"))
            .add_source(File::with_name("src/config/local.toml").required(false))
            .build()?;
        conf.try_deserialize()
    }
}

impl FromRef<AppState> for Arc<Configuration> {
    fn from_ref(state: &AppState) -> Arc<Configuration> {
        Arc::clone(&state.config)
    }
}
