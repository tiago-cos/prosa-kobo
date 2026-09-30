use crate::{
    books::{Cover, Fixture},
    suite::{Ctx, SYNC, Section, StepResult, steps},
};

pub fn section() -> Section {
    Section {
        name: "sync",
        summary: "books added, replaced and deleted elsewhere",
        steps: steps![
            "New books appear on the device" => new_books,
            "Covers and authors arrive with them" => covers_and_authors,
            "A book downloads and opens" => opens,
            "A book added elsewhere arrives on the next sync" => added_later,
            "A file replaced elsewhere reaches the device" => replaced,
            "A book deleted elsewhere leaves the device" => deleted,
        ],
    }
}

async fn new_books(ctx: &mut Ctx) -> StepResult {
    ctx.library(&[Fixture::Lighthouse, Fixture::Orchard]).await
}

async fn covers_and_authors(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.confirm(&format!(
        "Is “The Lighthouse Keeper” by Ada Beacon, its cover {}, and “An Orchard Year” by Bram Cider, its cover {}?",
        Cover::Blue.describe(),
        Cover::Green.describe()
    ))
}

async fn opens(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.instruct(
        "On the Kobo",
        &["Open “The Lighthouse Keeper” and let it download"],
    )?;

    ctx.prompt
        .confirm(&format!(
            "Did it open on a cover page, {}, with chapter “1. Arrival” and its paragraph [1.1] on the page after?",
            Cover::Blue.describe()
        ))
}

async fn added_later(ctx: &mut Ctx) -> StepResult {
    ctx.add(Fixture::Tidepool, None).await?;
    ctx.prompt
        .instruct("On the Kobo", &["Go back to the Home screen", SYNC])?;

    ctx.prompt.confirm(&format!(
        "Is “Tidepool Notes” by Marina Shore now in My Books, its cover {}?",
        Cover::Purple.describe()
    ))
}

async fn replaced(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Orchard)?;
    let key = ctx.key().to_owned();
    ctx.api().delete_book(&book_id, &key).await?;
    ctx.prosa
        .stock(Fixture::OrchardRevised, Some(&book_id), &key)
        .await?;

    ctx.prompt.instruct(
        "On the Kobo",
        &[
            SYNC,
            "Open “An Orchard Year”, letting it download again if it asks",
        ],
    )?;

    ctx.prompt.confirm(
        "Does paragraph [1.1] begin “This is the revised edition.”, and is there a fourth chapter, “Afterword”?",
    )
}

async fn deleted(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Tidepool)?;
    ctx.api().delete_book(&book_id, ctx.key()).await?;
    ctx.forget(Fixture::Tidepool);

    ctx.prompt
        .instruct("On the Kobo", &["Go back to the Home screen", SYNC])?;

    ctx.prompt
        .confirm("Is “Tidepool Notes” gone from My Books, leaving the other two?")
}
