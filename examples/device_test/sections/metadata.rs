use crate::{
    books::{AWKWARD_TITLE, Cover, FILE_PUBLISHER, Fixture},
    suite::{Ctx, SYNC, Section, StepResult, steps},
};
use serde_json::json;

const REFRESH: &str =
    "Switch to another tab and back to My Books: the Kobo redraws a cover only when it comes back into view";

const DETAILS: &str =
    "Open the book's details: tap the ⋯ under its cover (or long-press it) and choose View Details";

pub fn section() -> Section {
    Section {
        name: "metadata",
        summary: "book details and covers changed elsewhere",
        steps: steps![
            "The book arrives" => library,
            "Its details arrive in full" => in_full,
            "Changed details reach the device" => changed,
            "A title JSON has to escape arrives intact" => awkward_title,
            "A replaced cover reaches the device" => replaced_cover,
            "A removed cover leaves the device" => removed_cover,
            "A cover added again reaches the device" => added_cover,
            "A book whose details were all removed stays readable" => removed_metadata,
        ],
    }
}

async fn library(ctx: &mut Ctx) -> StepResult {
    ctx.library(&[Fixture::Lighthouse]).await
}

async fn in_full(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.instruct("On the Kobo", &[DETAILS])?;

    ctx.prompt.confirm(&format!(
        "Do the details show the title “The Lighthouse Keeper”, the author Ada Beacon, the series “Device Tests” (book 1), \
         the publisher {FILE_PUBLISHER}, which the Kobo takes from the book's own file rather than from Prosa, \
         the publication date 3 February 2001, the ISBN 9780000000001 \
         and the description “Description one of The Lighthouse Keeper.”?"
    ))
}

async fn changed(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Lighthouse)?;
    let change = json!({
        "title": "The Lighthouse Keeper, Revised",
        "description": "Description two, changed elsewhere.",
        "series": { "title": "Device Tests", "number": 5 },
    });
    ctx.prosa.patch_metadata(&book_id, &change, ctx.key()).await?;

    ctx.prompt
        .instruct("On the Kobo", &["Go back to the Home screen", SYNC, DETAILS])?;

    ctx.prompt.confirm(
        "Is the title now “The Lighthouse Keeper, Revised”, the series book 5, and the description \
         “Description two, changed elsewhere.”?",
    )
}

async fn awkward_title(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Lighthouse)?;
    ctx.prosa
        .patch_metadata(&book_id, &json!({ "title": AWKWARD_TITLE }), ctx.key())
        .await?;

    ctx.prompt
        .instruct("On the Kobo", &["Go back to the Home screen", SYNC])?;

    ctx.prompt.confirm(&format!(
        "Does My Books show the title exactly as {AWKWARD_TITLE}?"
    ))
}

async fn replaced_cover(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Lighthouse)?;
    ctx.prosa
        .replace_cover(&book_id, Cover::Orange, ctx.key())
        .await?;

    ctx.prompt.instruct("On the Kobo", &[SYNC, REFRESH])?;

    ctx.prompt.confirm(&format!(
        "Is the book's cover now {}, rather than {}?",
        Cover::Orange.describe(),
        Cover::Blue.describe()
    ))
}

async fn removed_cover(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Lighthouse)?;
    ctx.prosa.delete_cover(&book_id, ctx.key()).await?;

    ctx.prompt.instruct("On the Kobo", &[SYNC, REFRESH])?;

    ctx.prompt
        .confirm("Is the triangle cover gone, replaced by the Kobo's own plain cover?")
}

async fn added_cover(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Lighthouse)?;
    ctx.prosa.add_cover(&book_id, Cover::Blue, ctx.key()).await?;

    ctx.prompt.instruct("On the Kobo", &[SYNC, REFRESH])?;

    ctx.prompt
        .confirm(&format!("Is the cover {} again?", Cover::Blue.describe()))
}

async fn removed_metadata(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Lighthouse)?;
    ctx.prosa.delete_metadata(&book_id, ctx.key()).await?;

    ctx.prompt.instruct("On the Kobo", &[SYNC, "Open the book"])?;

    ctx.prompt
        .confirm("Is the book still in My Books, now named “Untitled”, and did it open?")
}
