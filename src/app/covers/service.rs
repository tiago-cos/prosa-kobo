use crate::{
    app::{authentication::AuthError, devices},
    client::prosa::Client,
};
use sqlx::SqlitePool;

pub async fn download_cover(
    pool: &SqlitePool,
    client: &Client,
    book_id: &str,
    device_id: &str,
) -> Result<Vec<u8>, AuthError> {
    let api_key = match devices::service::get_linked_device(pool, &device_id).await {
        Some(device) => device.api_key,
        _ => Err(AuthError::InvalidToken)?,
    };

    let cover = client
        .download_cover(&book_id, &api_key)
        .expect("Download should not fail");

    Ok(cover)
}
