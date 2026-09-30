use super::{device_bookmark, in_place, set_state, status};
use crate::{
    books::{Fixture, MID_PARAGRAPH, PROSA_PARAGRAPH_END},
    suite::{CLOSE, Ctx, SYNC, Section, StepResult, fail, steps, until},
};
use prosa_kobo::client::state::ProsaReadingStatus;

const BOOK: Fixture = Fixture::Lighthouse;
const MARK: &str = "On the Home screen, tap the ⋯ under “The Lighthouse Keeper” (or long-press it)";

pub fn section() -> Section {
    Section {
        name: "reading",
        summary: "positions and reading status in both directions",
        steps: steps![
            "The book arrives" => library,
            "Opening the book reaches Prosa" => opened,
            "A position on the device reaches Prosa" => device_position,
            "A position set elsewhere reaches the device" => prosa_position,
            "A book moved elsewhere reopens with no second offer" => reopened,
            "A position mid-paragraph reaches the device" => mid_paragraph,
            "A position at the end of a paragraph reaches the device" => paragraph_end,
            "Finishing on the device reaches Prosa" => device_finished,
            "Marking unread on the device reaches Prosa" => device_unread,
            "A book marked unread opens with no offer to move" => unread_opened,
            "Finished elsewhere reaches the device" => prosa_finished,
            "Unread elsewhere reaches the device" => prosa_unread,
            "Reading elsewhere reaches a device holding the book unread" => prosa_reading,
        ],
    }
}

async fn library(ctx: &mut Ctx) -> StepResult {
    ctx.library(&[BOOK]).await
}

async fn expect_status(ctx: &mut Ctx, expected: ProsaReadingStatus) -> StepResult {
    let state = until!(ctx, {
        let state = ctx.state(BOOK).await?;
        if state.statistics.reading_status == expected {
            Ok(state)
        } else {
            Err(format!(
                "Prosa has the book as {}, not {}",
                status(state.statistics.reading_status),
                status(expected)
            ))
        }
    });

    let position = state
        .location
        .as_deref()
        .map_or_else(|| "none".to_owned(), |location| ctx.book(BOOK).describe(location));
    ctx.prompt.detail("Prosa's position", &position);

    Ok(())
}

async fn opened(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.instruct(
        "On the Kobo",
        &["Open “The Lighthouse Keeper”, letting it download", CLOSE, SYNC],
    )?;

    expect_status(ctx, ProsaReadingStatus::Reading).await
}

async fn device_position(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            "Open “The Lighthouse Keeper”",
            "Using the table of contents, go to chapter 2 “The Keeper”",
            "Turn pages until the one on which paragraph [2.6] begins",
            CLOSE,
            SYNC,
        ],
    )?;

    let location = until!(ctx, {
        let location = ctx.state(BOOK).await?.location;
        in_place(ctx.book(BOOK), location.as_deref(), 1, 3..=6).map(|()| location)
    });

    let description = ctx.book(BOOK).describe(location.as_deref().unwrap_or_default());
    ctx.prompt.detail("Prosa's position", &description);
    ctx.prompt
        .confirm("Is that text at the top of the page you left, or just above it?")
}

/// Sync leaves a book being read alone; the device fetches the position when
/// the book is opened and offers it.
async fn move_to(ctx: &mut Ctx, location: String, landmark: &str) -> StepResult {
    let span = device_bookmark(ctx, BOOK, &location)
        .await?
        .ok_or_else(|| fail("prosa-kobo would offer the device no bookmark for this position"))?;
    ctx.prompt.detail("The device will be offered", &span);
    set_state(ctx, BOOK, Some(location), ProsaReadingStatus::Reading).await?;

    ctx.prompt.instruct(
        "On the Kobo",
        &[
            SYNC,
            "Open “The Lighthouse Keeper”. It should open where the Kobo last had it, then offer to go to a newer page: accept",
        ],
    )?;

    ctx.prompt.confirm(&format!(
        "Did it offer to move, and is it now on the page showing {landmark}?"
    ))
}

async fn prosa_position(ctx: &mut Ctx) -> StepResult {
    let location = ctx.book(BOOK).position("[5.4]");
    ctx.prompt.instruct("On the Kobo", &[CLOSE])?;

    move_to(
        ctx,
        location,
        "the start of paragraph [5.4], in chapter 5 “Storms”",
    )
    .await
}

async fn reopened(ctx: &mut Ctx) -> StepResult {
    ctx.prompt
        .instruct("On the Kobo", &[CLOSE, "Open “The Lighthouse Keeper” again"])?;

    ctx.prompt
        .confirm("Did it open on the same page, without offering to go to another page?")
}

async fn mid_paragraph(ctx: &mut Ctx) -> StepResult {
    let location = ctx.book(BOOK).position(MID_PARAGRAPH);
    ctx.prompt.instruct("On the Kobo", &[CLOSE])?;

    move_to(
        ctx,
        location,
        "“The ferry horn sounded across the bay.”, which ends paragraph [6.4]",
    )
    .await
}

async fn paragraph_end(ctx: &mut Ctx) -> StepResult {
    let location = ctx.book(BOOK).position_after(PROSA_PARAGRAPH_END);
    ctx.prompt.instruct("On the Kobo", &[CLOSE])?;

    move_to(
        ctx,
        location,
        "“…and the fog rolled in”, the very end of paragraph [5.8]",
    )
    .await
}

async fn device_finished(ctx: &mut Ctx) -> StepResult {
    ctx.prompt
        .instruct("On the Kobo", &[CLOSE, MARK, "Choose Mark as finished", SYNC])?;

    expect_status(ctx, ProsaReadingStatus::Read).await
}

async fn device_unread(ctx: &mut Ctx) -> StepResult {
    ctx.prompt
        .instruct("On the Kobo", &[MARK, "Choose Mark as unread", SYNC])?;

    expect_status(ctx, ProsaReadingStatus::Unread).await?;
    if let Some(location) = ctx.state(BOOK).await?.location {
        return Err(fail(format!(
            "The book is unread, but Prosa kept its position, {}",
            ctx.book(BOOK).describe(&location)
        )));
    }

    Ok(())
}

async fn unread_opened(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            "Open “The Lighthouse Keeper”, noting whether it offers to go to another page",
            CLOSE,
        ],
    )?;

    ctx.prompt
        .confirm("Did it open without offering to go to another page?")
}

async fn prosa_finished(ctx: &mut Ctx) -> StepResult {
    set_state(ctx, BOOK, None, ProsaReadingStatus::Read).await?;

    ctx.prompt.instruct("On the Kobo", &[SYNC])?;

    ctx.prompt
        .confirm("Is “The Lighthouse Keeper” marked Finished in My Books?")
}

async fn prosa_unread(ctx: &mut Ctx) -> StepResult {
    set_state(ctx, BOOK, None, ProsaReadingStatus::Unread).await?;

    ctx.prompt.instruct("On the Kobo", &[SYNC])?;

    ctx.prompt
        .confirm("Is the book marked unread (New, no progress shown)?")
}

/// The device does not fetch the state of a book it holds as unread, so sync
/// moves it without asking.
async fn prosa_reading(ctx: &mut Ctx) -> StepResult {
    let location = ctx.book(BOOK).position("[2.6]");
    set_state(ctx, BOOK, Some(location), ProsaReadingStatus::Reading).await?;

    ctx.prompt
        .instruct("On the Kobo", &[SYNC, "Open “The Lighthouse Keeper”"])?;
    ctx.prompt.confirm(
        "Did it open straight on the page showing the start of paragraph [2.6], in chapter 2 “The Keeper”, \
         without offering to go to another page?",
    )?;

    ctx.prompt.instruct("On the Kobo", &[CLOSE])?;
    ctx.prompt
        .confirm("Back on the Home screen, does the book show as being read, with its progress?")
}
