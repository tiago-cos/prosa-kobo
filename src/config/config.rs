use crate::app::AppState;
use axum::extract::FromRef;
use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Server {
    pub bind: Bind,
    pub public: Option<Public>,
}

#[derive(Deserialize)]
pub struct Public {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Bind {
    pub host: String,
    pub port: u16,
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
        let default_config_path =
            std::env::var("DEFAULT_CONFIGURATION").unwrap_or_else(|_| "src/config/default.toml".to_string());

        let local_config_path =
            std::env::var("LOCAL_CONFIGURATION").unwrap_or_else(|_| "src/config/local.toml".to_string());

        let conf = Config::builder()
            .add_source(File::with_name(&default_config_path))
            .add_source(File::with_name(&local_config_path).required(false))
            .add_source(config::Environment::default().separator("__"))
            .build()?;

        conf.try_deserialize()
    }
}

impl FromRef<AppState> for Arc<Configuration> {
    fn from_ref(state: &AppState) -> Arc<Configuration> {
        Arc::clone(&state.config)
    }
}
