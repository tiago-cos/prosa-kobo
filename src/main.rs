#![allow(clippy::unreadable_literal)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::module_inception)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use config::Configuration;

mod app;
mod client;
mod config;
mod database;

#[tokio::main]
async fn main() {
    let config = Configuration::new().unwrap();
    let db_pool = database::debug_init(&config.database.file_path).await;

    app::run(config, db_pool).await;
}
