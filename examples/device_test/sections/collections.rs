use crate::{
    books::Fixture,
    suite::{Ctx, SYNC, Section, StepEnd, StepResult, fail, steps, until},
};
use std::collections::BTreeSet;

const COLLECTIONS: &str = "Open My Books, then Collections";

pub fn section() -> Section {
    Section {
        name: "collections",
        summary: "collections and Prosa shelves in both directions",
        steps: steps![
            "The books arrive" => library,
            "A collection made on the device reaches Prosa" => device_created,
            "A collection renamed on the device reaches Prosa" => device_renamed,
            "A book taken out on the device leaves the shelf" => device_removed,
            "A book put in on the device joins the shelf" => device_added,
            "A collection deleted on the device leaves Prosa" => device_deleted,
            "A shelf made elsewhere reaches the device" => prosa_created,
            "A shelf renamed elsewhere reaches the device" => prosa_renamed,
            "A book taken off the shelf elsewhere leaves the collection" => prosa_removed,
            "A book put on the shelf elsewhere joins the collection" => prosa_added,
            "A shelf deleted elsewhere leaves the device" => prosa_deleted,
            "A collection named like a shelf made elsewhere joins it" => clashing_names,
            "A book deleted elsewhere leaves its collection" => book_deleted,
        ],
    }
}

async fn library(ctx: &mut Ctx) -> StepResult {
    ctx.library(&[Fixture::Lighthouse, Fixture::Orchard, Fixture::Tidepool])
        .await
}

struct Shelf {
    id: String,
    name: String,
    books: BTreeSet<String>,
}

async fn shelves(ctx: &Ctx) -> Result<Vec<Shelf>, StepEnd> {
    let mut shelves = Vec::new();
    for id in ctx.prosa.shelves_of(&ctx.account.username, ctx.key()).await? {
        let name = ctx.api().get_shelf_metadata(&id, ctx.key()).await?.name;
        let books = ctx
            .api()
            .list_books_in_shelf(&id, ctx.key())
            .await?
            .into_iter()
            .collect();
        shelves.push(Shelf { id, name, books });
    }

    Ok(shelves)
}

fn ids(ctx: &Ctx, fixtures: &[Fixture]) -> Result<BTreeSet<String>, StepEnd> {
    fixtures.iter().map(|fixture| ctx.id(*fixture)).collect()
}

fn titles(fixtures: &[Fixture]) -> String {
    let titles: Vec<String> = fixtures.iter().map(|f| format!("“{}”", f.title())).collect();

    if titles.is_empty() {
        "nothing".to_owned()
    } else {
        titles.join(" and ")
    }
}

async fn expect_shelf(ctx: &mut Ctx, name: &str, fixtures: &[Fixture]) -> Result<String, StepEnd> {
    let expected = ids(ctx, fixtures)?;

    let shelf_id = until!(ctx, {
        let named: Vec<Shelf> = shelves(ctx)
            .await?
            .into_iter()
            .filter(|shelf| shelf.name == name)
            .collect();
        match named.as_slice() {
            [] => Err(format!("Prosa has no shelf named “{name}”")),
            [shelf] if shelf.books == expected => Ok(shelf.id.clone()),
            [shelf] => Err(format!(
                "“{name}” holds {} books in Prosa, not {}",
                shelf.books.len(),
                titles(fixtures)
            )),
            _ => Err(format!("Prosa has {} shelves named “{name}”", named.len())),
        }
    });

    Ok(shelf_id)
}

async fn device_created(ctx: &mut Ctx) -> StepResult {
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            COLLECTIONS,
            "Create a collection named “Kobo Shelf”",
            "Add “The Lighthouse Keeper” and “An Orchard Year” to it",
            SYNC,
        ],
    )?;

    let shelf_id = expect_shelf(ctx, "Kobo Shelf", &[Fixture::Lighthouse, Fixture::Orchard]).await?;
    ctx.remember("device shelf", &shelf_id);

    Ok(())
}

async fn device_renamed(ctx: &mut Ctx) -> StepResult {
    ctx.recall("device shelf")?;
    ctx.prompt.instruct(
        "On the Kobo",
        &[COLLECTIONS, "Rename “Kobo Shelf” to “Kobo Shelf Renamed”", SYNC],
    )?;

    expect_shelf(
        ctx,
        "Kobo Shelf Renamed",
        &[Fixture::Lighthouse, Fixture::Orchard],
    )
    .await
    .map(drop)
}

async fn device_removed(ctx: &mut Ctx) -> StepResult {
    ctx.recall("device shelf")?;
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            COLLECTIONS,
            "Open “Kobo Shelf Renamed” and take “An Orchard Year” out of it",
            SYNC,
        ],
    )?;

    expect_shelf(ctx, "Kobo Shelf Renamed", &[Fixture::Lighthouse])
        .await
        .map(drop)
}

async fn device_added(ctx: &mut Ctx) -> StepResult {
    ctx.recall("device shelf")?;
    ctx.prompt.instruct(
        "On the Kobo",
        &[COLLECTIONS, "Add “Tidepool Notes” to “Kobo Shelf Renamed”", SYNC],
    )?;

    expect_shelf(
        ctx,
        "Kobo Shelf Renamed",
        &[Fixture::Lighthouse, Fixture::Tidepool],
    )
    .await
    .map(drop)
}

async fn device_deleted(ctx: &mut Ctx) -> StepResult {
    let shelf_id = ctx.recall("device shelf")?;
    ctx.prompt.instruct(
        "On the Kobo",
        &[COLLECTIONS, "Delete the collection “Kobo Shelf Renamed”", SYNC],
    )?;

    until!(ctx, {
        if shelves(ctx).await?.iter().any(|shelf| shelf.id == shelf_id) {
            Err("Prosa still has the shelf".to_owned())
        } else {
            Ok(())
        }
    });

    for fixture in [Fixture::Lighthouse, Fixture::Tidepool] {
        if !ctx.prosa.book_exists(&ctx.id(fixture)?, ctx.key()).await? {
            return Err(fail(format!(
                "Deleting the collection deleted “{}” too",
                fixture.title()
            )));
        }
    }

    Ok(())
}

fn confirm_collection(ctx: &mut Ctx, name: &str, fixtures: &[Fixture]) -> StepResult {
    ctx.prompt.instruct("On the Kobo", &[SYNC, COLLECTIONS])?;

    ctx.prompt.confirm(&format!(
        "Is there one collection named “{name}”, holding {}?",
        titles(fixtures)
    ))
}

async fn prosa_created(ctx: &mut Ctx) -> StepResult {
    let key = ctx.key().to_owned();
    let shelf_id = ctx.api().create_shelf("Prosa Shelf", None, None, &key).await?;
    for fixture in [Fixture::Orchard, Fixture::Tidepool] {
        ctx.api()
            .add_book_to_shelf(&shelf_id, &ctx.id(fixture)?, &key)
            .await?;
    }
    ctx.remember("prosa shelf", &shelf_id);

    confirm_collection(ctx, "Prosa Shelf", &[Fixture::Orchard, Fixture::Tidepool])
}

async fn prosa_renamed(ctx: &mut Ctx) -> StepResult {
    let shelf_id = ctx.recall("prosa shelf")?;
    ctx.api()
        .update_shelf_name(&shelf_id, "Prosa Shelf Renamed", ctx.key())
        .await?;

    confirm_collection(ctx, "Prosa Shelf Renamed", &[Fixture::Orchard, Fixture::Tidepool])
}

async fn prosa_removed(ctx: &mut Ctx) -> StepResult {
    let shelf_id = ctx.recall("prosa shelf")?;
    ctx.api()
        .delete_book_from_shelf(&shelf_id, &ctx.id(Fixture::Orchard)?, ctx.key())
        .await?;

    confirm_collection(ctx, "Prosa Shelf Renamed", &[Fixture::Tidepool])
}

async fn prosa_added(ctx: &mut Ctx) -> StepResult {
    let shelf_id = ctx.recall("prosa shelf")?;
    ctx.api()
        .add_book_to_shelf(&shelf_id, &ctx.id(Fixture::Lighthouse)?, ctx.key())
        .await?;

    confirm_collection(
        ctx,
        "Prosa Shelf Renamed",
        &[Fixture::Lighthouse, Fixture::Tidepool],
    )
}

async fn prosa_deleted(ctx: &mut Ctx) -> StepResult {
    let shelf_id = ctx.recall("prosa shelf")?;
    ctx.api().delete_shelf(&shelf_id, ctx.key()).await?;

    ctx.prompt.instruct("On the Kobo", &[SYNC, COLLECTIONS])?;

    ctx.prompt
        .confirm("Is “Prosa Shelf Renamed” gone, with all three books still in My Books?")
}

async fn clashing_names(ctx: &mut Ctx) -> StepResult {
    let key = ctx.key().to_owned();
    let shelf_id = ctx.api().create_shelf("Shared Shelf", None, None, &key).await?;
    ctx.api()
        .add_book_to_shelf(&shelf_id, &ctx.id(Fixture::Tidepool)?, &key)
        .await?;

    ctx.prompt.instruct(
        "On the Kobo",
        &[
            "Do not sync until told to",
            COLLECTIONS,
            "Create a collection named “Shared Shelf” and add “The Lighthouse Keeper” to it",
            SYNC,
        ],
    )?;

    let merged = expect_shelf(ctx, "Shared Shelf", &[Fixture::Lighthouse, Fixture::Tidepool]).await?;
    if merged != shelf_id {
        return Err(fail("The device's collection became a shelf of its own"));
    }
    ctx.remember("shared shelf", &shelf_id);

    confirm_collection(ctx, "Shared Shelf", &[Fixture::Lighthouse, Fixture::Tidepool])
}

async fn book_deleted(ctx: &mut Ctx) -> StepResult {
    ctx.recall("shared shelf")?;
    ctx.api()
        .delete_book(&ctx.id(Fixture::Tidepool)?, ctx.key())
        .await?;
    ctx.forget(Fixture::Tidepool);

    confirm_collection(ctx, "Shared Shelf", &[Fixture::Lighthouse])
}
