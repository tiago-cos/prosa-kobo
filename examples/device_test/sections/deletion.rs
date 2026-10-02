use crate::{
    books::Fixture,
    suite::{CLOSE, Ctx, SYNC, Section, StepResult, fail, steps, until},
};

const REMOVE: &str =
    "On the Home screen or in the Books tab, tap the ⋯ under its cover (or long-press it) and choose Remove";

pub fn section() -> Section {
    Section {
        name: "deletion",
        summary: "books removed on the device and deleted elsewhere",
        steps: steps![
            "The books arrive" => library,
            "Removing only the download keeps the book in Prosa" => download_removed,
            "Removing a book from My Books deletes it in Prosa" => device_deleted,
            "A book deleted elsewhere leaves the device" => prosa_deleted,
            "A downloaded book deleted elsewhere leaves the device" => prosa_deleted_downloaded,
            "A deleted book uploaded again under its id comes back" => restored,
        ],
    }
}

async fn library(ctx: &mut Ctx) -> StepResult {
    ctx.library(&[Fixture::Lighthouse, Fixture::Orchard, Fixture::Tidepool])
        .await
}

async fn download_removed(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            "Open “An Orchard Year”, letting it download",
            CLOSE,
            &format!("{REMOVE} for “An Orchard Year”"),
            "Choose to remove only the download, not to remove it from My Books",
            SYNC,
        ],
    )?;

    if !ctx
        .prosa
        .book_exists(&ctx.id(Fixture::Orchard)?, ctx.key())
        .await?
    {
        ctx.forget(Fixture::Orchard);
        return Err(fail("Removing the download deleted the book from Prosa"));
    }
    ctx.prompt.good("Prosa still has the book");

    ctx.prompt
        .confirm("Is “An Orchard Year” still in the Books tab, shown as not downloaded?")
}

async fn device_deleted(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Tidepool)?;
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            &format!("{REMOVE} for “Tidepool Notes”"),
            "Choose to remove it from My Books",
            SYNC,
        ],
    )?;

    until!(ctx, {
        if ctx.prosa.book_exists(&book_id, ctx.key()).await? {
            Err("Prosa still has “Tidepool Notes”".to_owned())
        } else {
            Ok(())
        }
    });
    ctx.forget(Fixture::Tidepool);

    Ok(())
}

async fn prosa_deleted(ctx: &mut Ctx) -> StepResult {
    ctx.api()
        .delete_book(&ctx.id(Fixture::Orchard)?, ctx.key())
        .await?;
    ctx.forget(Fixture::Orchard);

    ctx.prompt.instruct("On the Kobo", &[SYNC])?;

    ctx.prompt
        .confirm("Is “An Orchard Year” gone from the Books tab?")
}

async fn prosa_deleted_downloaded(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.id(Fixture::Lighthouse)?;
    ctx.remember("deleted book", &book_id);
    ctx.prompt.instruct(
        "On the Kobo",
        &["Open “The Lighthouse Keeper”, letting it download", CLOSE],
    )?;

    ctx.api().delete_book(&book_id, ctx.key()).await?;
    ctx.forget(Fixture::Lighthouse);

    ctx.prompt.instruct("On the Kobo", &[SYNC])?;

    ctx.prompt
        .confirm("Is “The Lighthouse Keeper” gone from the Books tab, download and all?")
}

async fn restored(ctx: &mut Ctx) -> StepResult {
    let book_id = ctx.recall("deleted book")?;
    ctx.add(Fixture::Lighthouse, Some(&book_id)).await?;

    ctx.prompt
        .instruct("On the Kobo", &[SYNC, "Open “The Lighthouse Keeper”"])?;

    ctx.prompt
        .confirm("Is “The Lighthouse Keeper” back in the Books tab, and does it open?")
}
