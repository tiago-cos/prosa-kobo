use super::{
    data,
    models::{DeviceState, Location, RatingResponse, ReadingState, UpdateStateResponse, prosa_rating},
};
use crate::{
    app::{error::KoboError, kepub, kobo_time},
    client::{ProsaReadingStatus, prosa::ProsaApi},
    database::pool,
};

async fn prosa_state(client: &dyn ProsaApi, book_id: &str, api_key: &str) -> Result<ReadingState, KoboError> {
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

pub async fn translate_get_state(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
    device_id: &str,
) -> Result<ReadingState, KoboError> {
    let state = prosa_state(client, book_id, api_key).await?;
    let held = data::get_device_state(pool(), device_id, book_id).await?;

    if held == Some(DeviceState::of(&state)) {
        return Ok(state.dated(&kobo_time::epoch()));
    }

    Ok(state)
}

pub async fn translate_book_state(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
    device_id: &str,
) -> Result<ReadingState, KoboError> {
    let state = prosa_state(client, book_id, api_key).await?;
    data::set_device_state(pool(), device_id, book_id, &DeviceState::of(&state)).await?;

    Ok(state)
}

pub async fn translate_changed_state(
    client: &dyn ProsaApi,
    book_id: &str,
    api_key: &str,
    device_id: &str,
) -> Result<Option<ReadingState>, KoboError> {
    let state = prosa_state(client, book_id, api_key).await?;
    let sent = DeviceState::of(&state);
    let held = data::get_device_state(pool(), device_id, book_id).await?;

    if !sent.is_news_to(held.as_ref()) {
        return Ok(None);
    }

    data::set_device_state(pool(), device_id, book_id, &sent).await?;

    Ok(Some(state))
}

pub async fn forget_book(device_id: &str, book_id: &str) -> Result<(), KoboError> {
    data::remove_device_state(pool(), device_id, book_id).await?;

    Ok(())
}

pub async fn translate_update_state(
    client: &dyn ProsaApi,
    book_id: &str,
    state: &ReadingState,
    api_key: &str,
    device_id: &str,
) -> Result<UpdateStateResponse, KoboError> {
    let status: ProsaReadingStatus = state.status_info.status.into();

    if status == ProsaReadingStatus::Reading {
        let location = match state.current_bookmark.location.as_ref().map(Location::position) {
            Some(position) => {
                let kepub = kepub::get_kepub(client, book_id, api_key).await?;
                kepub::to_prosa_location(&kepub, &position)
            }
            None => None,
        };

        client
            .patch_state(book_id, location.as_deref(), status, api_key)
            .await?;
    } else {
        let mut stored = client.fetch_state(book_id, api_key).await?;
        stored.location = None;
        stored.statistics.reading_status = status;
        client.replace_state(book_id, &stored, api_key).await?;
    }

    data::set_device_state(pool(), device_id, book_id, &DeviceState::of(state)).await?;

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
