use crate::{
    app::{covers::data, covers::models::COVER_TOKEN_SIZE, error::KoboError},
    client::prosa::ProsaApi,
};
use base64::{Engine, prelude::BASE64_URL_SAFE};
use image::{ImageError, ImageFormat, ImageReader, imageops::FilterType};
use rand::RngCore;
use sqlx::SqlitePool;
use std::io::Cursor;

pub fn download_cover(client: &dyn ProsaApi, book_id: &str, api_key: &str) -> Result<Vec<u8>, KoboError> {
    let cover = client.download_cover(book_id, api_key)?;

    Ok(cover)
}

pub fn resize_cover(cover: &Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>, ImageError> {
    let image = ImageReader::new(Cursor::new(cover))
        .with_guessed_format()?
        .decode()?
        .resize_exact(width, height, FilterType::Nearest);

    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, ImageFormat::Jpeg)?;

    Ok(output.into_inner())
}

pub async fn get_token(pool: &SqlitePool, book_id: &str, device_id: &str) -> String {
    match data::get_token(pool, device_id, book_id).await {
        Some(token) => token,
        None => update_token(pool, book_id, device_id).await,
    }
}

/// The Kobo refetches a cover only when its id changes, so the id carries a
/// value that is rotated whenever the cover does.
pub async fn update_token(pool: &SqlitePool, book_id: &str, device_id: &str) -> String {
    let mut bytes = vec![0u8; COVER_TOKEN_SIZE];
    rand::rng().fill_bytes(&mut bytes);
    let token = BASE64_URL_SAFE.encode(bytes);

    data::delete_token(pool, book_id, device_id).await;
    data::add_token(pool, book_id, &token, device_id).await;

    token
}
