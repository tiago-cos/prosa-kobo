use super::models::{Location, ReadingState, UpdateStateResponse};
use crate::{
    app::{
        error::KoboError,
        kepub::{self, KoboPosition},
        state::models::RatingResponse,
    },
    client::{ProsaReadingStatus, prosa::ProsaApi},
};

pub async fn translate_get_state(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<ReadingState, KoboError> {
    let state_response = client.fetch_state(book_id, api_key).await?;

    let status = match state_response.statistics.reading_status {
        ProsaReadingStatus::Read => "Finished",
        ProsaReadingStatus::Unread => "ReadyToRead",
        ProsaReadingStatus::Reading => "Reading",
    };

    // A bookmark we cannot translate is no bookmark: the device gets the book
    // without a reading position rather than a broken one.
    let position = match state_response.location {
        Some(location) => {
            let kepub = kepub::get_kepub(client, book_id, api_key).await?;
            kepub::to_kobo_position(&kepub, &location)
        }
        None => None,
    };

    let state = ReadingState::new(
        book_id,
        status,
        position.as_ref().map(|position| position.span.clone()),
        position.map(|position| position.chapter),
    );

    Ok(state)
}

pub async fn translate_update_state(
    client: &dyn ProsaApi,
    book_id: &str,
    state: &ReadingState,
    api_key: &str,
) -> Result<UpdateStateResponse, KoboError> {
    let status = match state.status_info.status.as_str() {
        "Finished" => ProsaReadingStatus::Read,
        "ReadyToRead" => ProsaReadingStatus::Unread,
        _ => ProsaReadingStatus::Reading,
    };

    let location = match &state.current_bookmark.location {
        Some(location) => {
            let kepub = kepub::get_kepub(client, book_id, api_key).await?;
            kepub::to_prosa_location(&kepub, &bookmark_position(location))
        }
        None => None,
    };

    client
        .patch_state(book_id, location.as_deref(), status, api_key)
        .await?;

    Ok(UpdateStateResponse::success(book_id))
}

pub async fn translate_update_rating(
    client: &dyn ProsaApi,
    book_id: &str,
    rating: u8,
    api_key: &str,
) -> Result<(), KoboError> {
    client.update_rating(book_id, rating, api_key).await?;

    Ok(())
}

pub async fn translate_get_rating(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<RatingResponse, KoboError> {
    let rating = client.fetch_rating(book_id, api_key).await?;

    Ok(RatingResponse::new(book_id, rating))
}

// The device names the chapter relative to the book file it downloaded, as
// `<book>.kepub.epub!!OEBPS/chapter.xhtml`; only the tail names a document.
fn bookmark_position(location: &Location) -> KoboPosition {
    let chapter = match location.source.split_once("!!") {
        Some((_, source)) => source,
        None => location.source.as_str(),
    };

    KoboPosition::new(chapter, &location.value, 0)
}
