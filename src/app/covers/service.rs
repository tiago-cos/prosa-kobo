use crate::{app::covers::data, app::error::KoboError, client::prosa::ProsaApi, database::pool};
use image::{ImageError, ImageFormat, ImageReader, imageops::FilterType};
use std::io::Cursor;

pub async fn download_cover(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<Vec<u8>, KoboError> {
    let cover = client.download_cover(book_id, api_key).await?;

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

/// The Kobo refetches a cover only when its id changes, so the id carries a
/// version that moves whenever the cover does.
pub async fn get_version(device_id: &str, book_id: &str) -> Result<i64, KoboError> {
    let version = data::get_version(pool(), device_id, book_id).await?;

    Ok(version.unwrap_or_default())
}

pub async fn bump_version(device_id: &str, book_id: &str) -> Result<(), KoboError> {
    data::bump_version(pool(), device_id, book_id).await?;

    Ok(())
}
