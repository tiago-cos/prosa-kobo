use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Configuration {
    pub server: Server,
    pub database: Database,
    pub prosa: Prosa,
}

#[derive(Default, Deserialize)]
#[serde(default)]
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
#[serde(default)]
pub struct Bind {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Prosa {
    pub host: String,
    pub port: u16,
    pub scheme: String,
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct Database {
    pub file_path: String,
}

impl Default for Bind {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 5001,
        }
    }
}

impl Default for Prosa {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 5000,
            scheme: "http".to_string(),
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            file_path: "persistence/database.db".to_string(),
        }
    }
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = std::env::var("CONFIGURATION").unwrap_or_else(|_| "configuration.toml".to_string());

        let conf = Config::builder()
            .add_source(File::with_name(&config_path).required(false))
            .add_source(config::Environment::default().separator("__"))
            .build()?;

        conf.try_deserialize()
    }
}
