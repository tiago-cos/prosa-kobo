#![allow(clippy::unreadable_literal)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::module_inception)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::app::generate_jwt_secret;
use config::Configuration;
use std::{io::Error, path::Path, sync::LazyLock};
use tokio::fs;
mod app;
mod client;
mod config;
mod database;

static CONFIG: LazyLock<Configuration> =
    LazyLock::new(|| Configuration::new().expect("Failed to load configuration"));

#[tokio::main]
async fn main() {
    create_parent_dir(&CONFIG.database.file_path).await.unwrap();
    create_parent_dir(&CONFIG.auth.jwt_key_path).await.unwrap();
    generate_jwt_secret(&CONFIG.auth.jwt_key_path).await.unwrap();

    let db_pool = database::init(&CONFIG.database.file_path).await;

    println!(
        r"
 ───────────────────────────────────────────────────────
  ____                            _  __     _           
 |  _ \ _ __ ___  ___  __ _      | |/ /___ | |__   ___  
 | |_) | '__/ _ \/ __|/ _` | ___ | ' // _ \| '_ \ / _ \ 
 |  __/| | | (_) \__ \ (_| | ___ | . \ (_) | |_) | (_) |
 |_|   |_|  \___/|___/\__,_|     |_|\_\___/|_.__/ \___/ 

 ───────────────────────────────────────────────────────
        "
    );

    app::run(db_pool).await;
}

async fn create_parent_dir(path: &str) -> Result<(), Error> {
    let path = Path::new(path);

    if !path.exists()
        && let Some(parent) = path.parent()
    {
        fs::create_dir_all(parent).await?;
    }

    Ok(())
}
