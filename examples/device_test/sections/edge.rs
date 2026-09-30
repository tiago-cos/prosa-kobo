use super::{device_bookmark, in_place, is_element, set_state};
use crate::{
    books::{BLANK_CHAPTER, Fixture, PLATE_CHAPTER},
    suite::{CLOSE, Ctx, SYNC, Section, StepResult, fail, steps, until},
};
use prosa_kobo::client::state::ProsaReadingStatus;

const BOOK: Fixture = Fixture::Lighthouse;

pub fn section() -> Section {
    Section {
        name: "edge",
        summary: "books and positions with nothing to translate",
        steps: steps![
            "The book arrives" => library,
            "A book without details or cover arrives and opens" => bare_book,
            "A page that is only an image is saved as a position" => device_image_page,
            "A position on an image-only page reaches the device" => prosa_image_page,
            "A blank page does not break the saved position" => device_blank_page,
            "A position on a blank page is left out, not sent broken" => prosa_blank_page,
        ],
    }
}

async fn library(ctx: &mut Ctx) -> StepResult {
    ctx.library(&[BOOK]).await
}

async fn bare_book(ctx: &mut Ctx) -> StepResult {
    ctx.add(Fixture::Bare, None).await?;

    ctx.prompt.instruct(
        "On the Kobo",
        &[
            SYNC,
            "Find the new book in My Books: Prosa knows nothing about it, not even its title",
            "Open it",
        ],
    )?;

    ctx.prompt.confirm(
        "Is it listed as “Untitled”, and does it open at chapter “1. Only”, starting with paragraph [1.1]?",
    )
}

async fn device_image_page(ctx: &mut Ctx) -> StepResult {
    let expected = ctx.book(BOOK).element_location(PLATE_CHAPTER, &[0, 0]);

    ctx.prompt.instruct(
        "On the Kobo",
        &[
            "Open “The Lighthouse Keeper”, letting it download",
            "Using the table of contents, go to chapter 3 “Plate”: a page holding only a black circle",
            CLOSE,
            SYNC,
        ],
    )?;

    let location = until!(ctx, {
        let location = ctx.state(BOOK).await?.location;
        match location {
            Some(location) if location == expected => Ok(location),
            Some(location) => Err(format!(
                "Prosa has the book at {}, not at the image {expected}",
                ctx.book(BOOK).describe(&location)
            )),
            None => Err("Prosa has no position for the book".to_owned()),
        }
    });

    ctx.prompt.detail("Prosa's position", &location);

    Ok(())
}

async fn prosa_image_page(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            "Open the book again and go back to chapter 1 “Arrival”",
            CLOSE,
            SYNC,
        ],
    )?;

    until!(ctx, {
        let location = ctx.state(BOOK).await?.location;
        in_place(ctx.book(BOOK), location.as_deref(), 0, 0..=10)
    });

    let image = ctx.book(BOOK).element_location(PLATE_CHAPTER, &[0, 0]);
    let span = device_bookmark(ctx, BOOK, &image)
        .await?
        .ok_or_else(|| fail("prosa-kobo would offer the device no bookmark for the image"))?;
    ctx.prompt.detail("The device will be offered", &span);
    set_state(ctx, BOOK, Some(image), ProsaReadingStatus::Reading).await?;

    ctx.prompt.instruct(
        "On the Kobo",
        &[
            SYNC,
            "Open the book. It should open in chapter 1, then offer to go to a newer page: accept",
        ],
    )?;

    ctx.prompt
        .confirm("Did it offer to move, and is it now on the black circle of chapter 3 “Plate”?")
}

async fn device_blank_page(ctx: &mut Ctx) -> StepResult {
    let before = ctx.state(BOOK).await?.location;

    ctx.prompt.instruct(
        "On the Kobo",
        &[
            "Open the book and go to chapter 4 “Blank”, a page with nothing on it but a rule",
            CLOSE,
            SYNC,
        ],
    )?;

    let after = ctx.state(BOOK).await?.location;
    let book = ctx.book(BOOK);
    let description = match &after {
        Some(location) if after == before => format!("unchanged, {}", book.describe(location)),
        Some(location) => book.describe(location),
        None => "none".to_owned(),
    };
    if let Some(location) = &after {
        book.resolve(location).map_err(|problem| {
            fail(format!(
                "Prosa was left a position that does not resolve: {problem}"
            ))
        })?;
    }
    ctx.prompt.detail("Prosa's position", &description);

    ctx.prompt.confirm(
        "Did the Kobo sync without an error, and does reopening the book land on the blank page, \
         with no offer to go to the image?",
    )
}

async fn prosa_blank_page(ctx: &mut Ctx) -> StepResult {
    let blank = ctx.book(BOOK).element_location(BLANK_CHAPTER, &[0]);
    if !is_element(ctx.book(BOOK), &blank) {
        return Err(fail("The blank page's rule is not an element"));
    }

    if let Some(span) = device_bookmark(ctx, BOOK, &blank).await? {
        return Err(fail(format!(
            "prosa-kobo would send the device {span} for a page with no span on it"
        )));
    }
    ctx.prompt
        .good("prosa-kobo sends the book without a bookmark rather than a broken one");
    set_state(ctx, BOOK, Some(blank), ProsaReadingStatus::Reading).await?;

    ctx.prompt
        .instruct("On the Kobo", &[CLOSE, SYNC, "Open the book"])?;

    ctx.prompt
        .confirm("Did the Kobo sync and open the book without an error?")
}
