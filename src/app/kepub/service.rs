use super::models::{KepubCache, KepubError};
use crate::app::{ProsaClient, error::KoboError};
use kepub_rs::Converter;
use std::{io::Cursor, sync::Arc};

pub async fn get_kepub(
    cache: &Arc<KepubCache>,
    client: &ProsaClient,
    book_id: &str,
    api_key: &str,
) -> Result<Arc<[u8]>, KoboError> {
    if let Some(kepub) = cache.get(book_id) {
        return Ok(kepub);
    }

    let epub = client.download_book(book_id, api_key)?;

    let kepub: Arc<[u8]> = tokio::task::spawn_blocking(move || convert(&epub))
        .await
        .expect("Kepub conversion task failed")?
        .into();

    cache.insert(book_id, &kepub);

    Ok(kepub)
}

pub fn evict(cache: &Arc<KepubCache>, book_id: &str) {
    cache.evict(book_id);
}

fn convert(epub: &[u8]) -> Result<Vec<u8>, KepubError> {
    let mut kepub = Cursor::new(Vec::new());

    Converter::default()
        .convert(Cursor::new(epub), &mut kepub)
        .map_err(|_| KepubError::ConversionFailed)?;

    Ok(kepub.into_inner())
}
