use super::models::{KepubError, KoboPosition};
use crate::{
    app::{KEPUBS, error::KoboError},
    client::prosa::ProsaApi,
};
use kepub_rs::{Converter, epub_to_kobo_location, kobo_to_epub_location};
use log::warn;
use std::{io::Cursor, sync::Arc};

pub async fn get_kepub(client: &dyn ProsaApi, book_id: &str, api_key: &str) -> Result<Arc<[u8]>, KoboError> {
    if let Some(kepub) = KEPUBS.get(book_id) {
        return Ok(kepub);
    }

    let epub = client.download_book(book_id, api_key)?;

    let kepub: Arc<[u8]> = tokio::task::spawn_blocking(move || convert(&epub))
        .await
        .expect("Kepub conversion task failed")?
        .into();

    KEPUBS.insert(book_id, &kepub);

    Ok(kepub)
}

pub fn evict(book_id: &str) {
    KEPUBS.evict(book_id);
}

fn convert(epub: &[u8]) -> Result<Vec<u8>, KepubError> {
    let mut kepub = Cursor::new(Vec::new());

    Converter::default()
        .convert(Cursor::new(epub), &mut kepub)
        .map_err(|_| KepubError::ConversionFailed)?;

    Ok(kepub.into_inner())
}

pub fn to_prosa_location(kepub: &[u8], position: &KoboPosition) -> Option<String> {
    let kobo = format!("{}#{}:{}", position.chapter, position.span, position.offset);

    kobo_to_epub_location(Cursor::new(kepub), &kobo)
        .inspect_err(|e| warn!("Could not translate {kobo} into a Prosa location: {e}"))
        .ok()
}

pub fn to_kobo_position(kepub: &[u8], location: &str) -> Option<KoboPosition> {
    let kobo = epub_to_kobo_location(Cursor::new(kepub), location)
        .inspect_err(|e| warn!("Could not translate {location} into a Kobo position: {e}"))
        .ok()?;

    let (chapter, rest) = kobo.rsplit_once('#')?;
    let (span, offset) = rest.rsplit_once(':')?;

    Some(KoboPosition::new(chapter, span, offset.parse().ok()?))
}
