mod annotations;
mod collections;
mod deletion;
mod edge;
mod metadata;
mod ratings;
mod reading;
mod sync;

use crate::{
    books::Fixture,
    epub::{Book, Resolved},
    suite::{Ctx, Problem, Section, StepEnd, StepResult},
};
use prosa_kobo::{
    app::kepub,
    client::state::{ProsaReadingStatus, ProsaState, ProsaStatistics},
};
use serde_json::Value;
use std::ops::RangeInclusive;

pub fn all() -> Vec<Section> {
    vec![
        sync::section(),
        metadata::section(),
        reading::section(),
        ratings::section(),
        annotations::section(),
        collections::section(),
        deletion::section(),
        edge::section(),
    ]
}

/// Where a phrase sits, as the tester finds it: by chapter and by the
/// paragraph label every generated paragraph starts with.
pub fn whereabouts(book: &Book, phrase: &str) -> String {
    let location = book.position(phrase);
    let resolved = book.resolve(&location).expect("A located phrase resolves");
    let chapter = resolved.chapter().expect("A phrase sits in a chapter");

    format!(
        "chapter {} “{}”, paragraph [{}.{}]",
        chapter + 1,
        book.chapters[chapter].title,
        chapter + 1,
        resolved.block()
    )
}

pub fn in_place(
    book: &Book,
    location: Option<&str>,
    chapter: usize,
    blocks: RangeInclusive<usize>,
) -> Result<(), String> {
    let location = location.ok_or("Prosa has no position for the book")?;
    let resolved = book.resolve(location)?;
    let paragraphs = format!("[{0}.{1}]–[{0}.{2}]", chapter + 1, blocks.start(), blocks.end());

    if resolved.chapter() != Some(chapter) || !blocks.contains(&resolved.block()) {
        return Err(format!(
            "Prosa has the book at {}, not within {paragraphs}",
            book.describe(location)
        ));
    }

    Ok(())
}

pub fn is_element(book: &Book, location: &str) -> bool {
    matches!(book.resolve(location), Ok(Resolved::Element { .. }))
}

pub async fn set_state(
    ctx: &Ctx,
    fixture: Fixture,
    location: Option<String>,
    reading_status: ProsaReadingStatus,
) -> StepResult {
    let rating = ctx.state(fixture).await?.statistics.rating;
    let state = ProsaState {
        location,
        statistics: ProsaStatistics {
            rating,
            reading_status,
        },
    };

    ctx.replace_state(fixture, &state).await
}

pub async fn set_rating(ctx: &Ctx, fixture: Fixture, rating: Option<f32>) -> StepResult {
    let mut state = ctx.state(fixture).await?;
    state.statistics.rating = rating;

    ctx.replace_state(fixture, &state).await
}

pub async fn as_device(ctx: &Ctx, path: &str) -> Result<Value, Problem> {
    ctx.middleware.as_device(ctx.account.lookup_key(), path).await
}

/// The bookmark the device is offered for `location`, translated as prosa-kobo
/// translates it, against the kepub the device downloads. The state endpoint is
/// not asked, since answering records the state as the one the device holds.
pub async fn device_bookmark(ctx: &Ctx, fixture: Fixture, location: &str) -> Result<Option<String>, StepEnd> {
    let kepub = ctx
        .middleware
        .download_as_device(ctx.account.lookup_key(), &format!("/books/{}", ctx.id(fixture)?))
        .await?;

    Ok(kepub::to_kobo_position(&kepub, location).map(|position| position.span))
}

pub async fn device_annotation_ids(ctx: &Ctx, fixture: Fixture) -> Result<Vec<String>, StepEnd> {
    let annotations = as_device(ctx, &format!("/api/v3/content/{}/annotations", ctx.id(fixture)?)).await?;

    Ok(annotations["annotations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|annotation| annotation["id"].as_str().map(ToOwned::to_owned))
        .collect())
}

pub fn fraction(rating: Option<f32>) -> String {
    rating.map_or_else(|| "no rating".to_owned(), |rating| format!("{rating} stars"))
}

pub fn status(status: ProsaReadingStatus) -> &'static str {
    match status {
        ProsaReadingStatus::Unread => "Unread",
        ProsaReadingStatus::Reading => "Reading",
        ProsaReadingStatus::Read => "Read",
    }
}

pub fn quoted(phrase: &str) -> String {
    format!("“{}”", phrase.replace('\n', " ⏎ "))
}
