#![allow(clippy::unreadable_literal)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::module_inception)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use config::Configuration;
use std::path::Path;
use tokio::fs::create_dir_all;

mod app;
mod client;
mod config;
mod database;

#[tokio::main]
async fn main() {
    let config = Configuration::new().unwrap();

    assert!(
        config.auth.secret_key.len() >= 16,
        "secret_key must be configured and at least 16 characters long"
    );

    let database_path = Path::new(&config.database.file_path)
        .parent()
        .expect("Invalid database path");

    create_dir_all(database_path).await.unwrap();

    let db_pool = database::init(&config.database.file_path).await;

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

    app::run(config, db_pool).await;
}
