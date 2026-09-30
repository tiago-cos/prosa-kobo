use super::{as_device, fraction, set_rating};
use crate::{
    books::Fixture,
    suite::{Ctx, SYNC, Section, StepResult, fail, steps, until},
};

const BOOK: Fixture = Fixture::Lighthouse;
const DETAILS: &str =
    "Open the book's details: tap the ⋯ under its cover (or long-press it) and choose View Details";

pub fn section() -> Section {
    Section {
        name: "ratings",
        summary: "star ratings in both directions",
        steps: steps![
            "The book arrives" => library,
            "A rating on the device reaches Prosa" => device_rating,
            "A changed rating on the device reaches Prosa" => device_changed,
            "A rating set elsewhere reaches the device" => prosa_rating,
            "A fractional rating is rounded for the device" => prosa_fraction,
            "A rating removed elsewhere leaves the device" => prosa_removed,
        ],
    }
}

async fn library(ctx: &mut Ctx) -> StepResult {
    ctx.library(&[BOOK]).await
}

async fn expect_rating(ctx: &mut Ctx, stars: f32) -> StepResult {
    until!(ctx, {
        let rating = ctx.state(BOOK).await?.statistics.rating;
        if rating == Some(stars) {
            Ok(())
        } else {
            Err(format!(
                "Prosa has {}, not {}",
                fraction(rating),
                fraction(Some(stars))
            ))
        }
    });

    Ok(())
}

async fn device_rating(ctx: &mut Ctx) -> StepResult {
    ctx.prompt
        .instruct("On the Kobo", &[DETAILS, "Rate the book 4 stars", SYNC])?;

    expect_rating(ctx, 4.0).await
}

async fn device_changed(ctx: &mut Ctx) -> StepResult {
    ctx.prompt
        .instruct("On the Kobo", &[DETAILS, "Change the rating to 2 stars", SYNC])?;

    expect_rating(ctx, 2.0).await
}

async fn device_stars(ctx: &Ctx) -> Result<u64, crate::suite::StepEnd> {
    let book_id = ctx.id(BOOK)?;
    let reviews = as_device(ctx, &format!("/v1/user/reviews?ProductIds={book_id}")).await?;

    Ok(reviews["Items"][0]["Rating"].as_u64().unwrap_or_default())
}

async fn shown(ctx: &mut Ctx, rating: Option<f32>, stars: u64) -> StepResult {
    set_rating(ctx, BOOK, rating).await?;

    let sent = device_stars(ctx).await?;
    if sent != stars {
        return Err(fail(format!(
            "prosa-kobo would tell the device {sent} stars for {}, not {stars}",
            fraction(rating)
        )));
    }

    ctx.prompt
        .instruct("On the Kobo", &["Leave the book's details", SYNC, DETAILS])?;

    let expected = if stars == 0 {
        "no stars".to_owned()
    } else {
        format!("{stars} stars")
    };
    ctx.prompt.confirm(&format!("Do the details show {expected}?"))
}

async fn prosa_rating(ctx: &mut Ctx) -> StepResult {
    shown(ctx, Some(5.0), 5).await
}

async fn prosa_fraction(ctx: &mut Ctx) -> StepResult {
    shown(ctx, Some(3.5), 4).await
}

async fn prosa_removed(ctx: &mut Ctx) -> StepResult {
    shown(ctx, None, 0).await
}
