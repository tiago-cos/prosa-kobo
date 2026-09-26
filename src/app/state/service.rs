use super::models::{Location, RatingResponse, ReadingState, UpdateStateResponse, prosa_rating};
use crate::{
    app::{error::KoboError, kepub},
    client::prosa::ProsaApi,
};

pub async fn translate_get_state(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<ReadingState, KoboError> {
    let state_response = client.fetch_state(book_id, api_key).await?;

    // A bookmark we cannot translate is no bookmark: the device gets the book
    // without a reading position rather than a broken one.
    let position = match state_response.location {
        Some(location) => {
            let kepub = kepub::get_kepub(client, book_id, api_key).await?;
            kepub::to_kobo_position(&kepub, &location)
        }
        None => None,
    };

    Ok(ReadingState::new(
        book_id,
        state_response.statistics.reading_status.into(),
        position,
    ))
}

pub async fn translate_update_state(
    client: &dyn ProsaApi,
    book_id: &str,
    state: &ReadingState,
    api_key: &str,
) -> Result<UpdateStateResponse, KoboError> {
    let location = match state.current_bookmark.location.as_ref().map(Location::position) {
        Some(position) => {
            let kepub = kepub::get_kepub(client, book_id, api_key).await?;
            kepub::to_prosa_location(&kepub, &position)
        }
        None => None,
    };

    client
        .patch_state(
            book_id,
            location.as_deref(),
            state.status_info.status.into(),
            api_key,
        )
        .await?;

    Ok(UpdateStateResponse::success(book_id))
}

pub async fn translate_update_rating(
    client: &dyn ProsaApi,
    book_id: &str,
    rating: u8,
    api_key: &str,
) -> Result<(), KoboError> {
    let mut state = client.fetch_state(book_id, api_key).await?;
    state.statistics.rating = prosa_rating(rating);
    client.replace_state(book_id, &state, api_key).await?;

    Ok(())
}

pub async fn translate_get_rating(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
) -> Result<RatingResponse, KoboError> {
    let state = client.fetch_state(book_id, api_key).await?;

    Ok(RatingResponse::new(book_id, state.statistics.rating))
}
