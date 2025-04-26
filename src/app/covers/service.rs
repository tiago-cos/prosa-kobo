use crate::{
    app::{authentication::AuthError, devices, error::KoboError},
    client::prosa::Client,
};
use regex::Regex;
use sqlx::SqlitePool;

pub async fn download_cover(
    pool: &SqlitePool,
    client: &Client,
    book_id: &str,
    device_id: &str,
) -> Result<Vec<u8>, KoboError> {
    let api_key = match devices::service::get_linked_device(pool, &device_id).await {
        Some(device) => device.api_key,
        _ => Err(AuthError::InvalidToken)?,
    };

    let re = Regex::new(r"\[\[.*?\]\]").expect("Failed to create regex");
    let book_id = re.replace_all(book_id, "").to_string();

    let cover = client.download_cover(&book_id, &api_key)?;

    Ok(cover)
}
