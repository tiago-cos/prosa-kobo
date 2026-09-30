use crate::suite::Problem;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub prosa_url: String,
    pub username: String,
    pub password: String,
    pub user_id: String,
    /// The key the Kobo is linked with. Prosa leaves a key's own changes out
    /// of what it syncs to that key, so nothing the suite does is made with it.
    pub device_key: String,
    /// The key the suite acts with, standing in for every other device.
    pub other_key: String,
    pub device_id: String,
    pub api_endpoint: String,
}

impl Account {
    pub fn directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".device-test")
    }

    fn path() -> PathBuf {
        Self::directory().join("account.json")
    }

    pub fn load() -> Result<Option<Self>, Problem> {
        match fs::read_to_string(Self::path()) {
            Ok(content) => serde_json::from_str(&content)
                .map(Some)
                .map_err(|e| Problem::new(format!("{} is unreadable: {e}", Self::path().display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Problem::new(format!(
                "Failed to read {}: {e}",
                Self::path().display()
            ))),
        }
    }

    pub fn save(&self) -> Result<(), Problem> {
        fs::create_dir_all(Self::directory()).map_err(|e| Problem::new(e.to_string()))?;
        let content = serde_json::to_string_pretty(self).map_err(|e| Problem::new(e.to_string()))?;

        fs::write(Self::path(), content).map_err(|e| Problem::new(format!("Failed to save the account: {e}")))
    }

    pub fn forget() -> Result<(), Problem> {
        match fs::remove_file(Self::path()) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(Problem::new(e.to_string())),
            _ => Ok(()),
        }
    }

    pub fn lookup_key(&self) -> &str {
        self.api_endpoint.rsplit('/').next().unwrap_or_default()
    }
}
