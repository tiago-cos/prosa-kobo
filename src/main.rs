use config::Configuration;

mod app;
mod config;
mod database;

#[tokio::main]
async fn main() {
    let config = Configuration::new().unwrap();
    let db_pool = database::debug_init(&config.database.file_path).await;

    app::run(config, db_pool).await;
}
