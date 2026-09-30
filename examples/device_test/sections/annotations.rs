use super::{device_annotation_ids, quoted, whereabouts};
use crate::{
    books::{
        BLANK_CHAPTER, DEVICE_ACCENTED, DEVICE_ACROSS_FORMATTING, DEVICE_ACROSS_PARAGRAPHS, DEVICE_HIGHLIGHT,
        DEVICE_NOTE, DEVICE_ONE_CHARACTER, DEVICE_PARAGRAPH_END, Fixture, PROSA_ACROSS_PARAGRAPHS,
        PROSA_BESIDE_UNTRANSLATABLE, PROSA_HIGHLIGHT, PROSA_NOTE, PROSA_ONE_CHARACTER, PROSA_PARAGRAPH_END,
    },
    suite::{CLOSE, Ctx, SYNC, Section, StepEnd, StepResult, fail, steps, until},
};
use prosa_kobo::client::{ProsaAnnotation, ProsaAnnotationRequest};

const BOOK: Fixture = Fixture::Lighthouse;
const DEVICE_NOTE_TEXT: &str = "Written on the Kobo";
const DEVICE_NOTE_EDITED: &str = "Edited on the Kobo";
const PROSA_NOTE_TEXT: &str = "Written elsewhere";
const PROSA_NOTE_EDITED: &str = "Edited elsewhere";

pub fn section() -> Section {
    Section {
        name: "annotations",
        summary: "highlights and notes in both directions, down to the character",
        steps: steps![
            "The book arrives and opens" => library,
            "A highlight made on the device reaches Prosa" => device_highlight,
            "A note made on the device reaches Prosa" => device_note,
            "A note edited on the device reaches Prosa" => device_note_edited,
            "A highlight deleted on the device leaves Prosa" => device_deleted,
            "A one-character highlight reaches Prosa" => device_one_character,
            "A highlight ending a paragraph reaches Prosa" => device_paragraph_end,
            "A highlight across two paragraphs reaches Prosa" => device_across_paragraphs,
            "A highlight across italics reaches Prosa" => device_across_formatting,
            "A highlight of accented words reaches Prosa" => device_accented,
            "Highlights added elsewhere reach the device" => prosa_added,
            "A note edited elsewhere reaches the device" => prosa_note_edited,
            "A highlight deleted elsewhere leaves the device" => prosa_deleted,
            "Edge-case highlights added elsewhere reach the device" => prosa_edge_cases,
            "An untranslatable annotation is left out, not the rest" => prosa_untranslatable,
        ],
    }
}

async fn library(ctx: &mut Ctx) -> StepResult {
    ctx.library(&[BOOK]).await?;
    ctx.prompt.instruct(
        "On the Kobo",
        &["Open “The Lighthouse Keeper”, letting it download", CLOSE],
    )
}

fn covered(ctx: &Ctx, annotation: &ProsaAnnotation) -> String {
    ctx.book(BOOK)
        .text_between(&annotation.start_location, &annotation.end_location)
        .unwrap_or_else(|problem| format!("<{problem}>"))
}

async fn highlight_on_device(
    ctx: &mut Ctx,
    phrase: &str,
    how: &str,
    note: Option<&str>,
) -> Result<ProsaAnnotation, StepEnd> {
    let before: Vec<String> = ctx
        .annotations(BOOK)
        .await?
        .into_iter()
        .map(|annotation| annotation.annotation_id)
        .collect();

    let whereabouts = whereabouts(ctx.book(BOOK), phrase);
    let select = format!("Select exactly {} {how}", quoted(phrase));
    let action = match note {
        Some(note) => format!(
            "Confirm the words are highlighted, then tap the highlight, choose Add note, type “{note}” and save it"
        ),
        None => "Confirm the words are highlighted".to_owned(),
    };
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            &format!("Open “The Lighthouse Keeper” at {whereabouts}"),
            &select,
            &action,
            CLOSE,
            SYNC,
        ],
    )?;

    let annotation = until!(ctx, {
        let new: Vec<ProsaAnnotation> = ctx
            .annotations(BOOK)
            .await?
            .into_iter()
            .filter(|annotation| !before.contains(&annotation.annotation_id))
            .collect();

        match new.iter().find(|annotation| covered(ctx, annotation) == phrase) {
            Some(annotation) => Ok(annotation.clone()),
            None if new.is_empty() => Err("Prosa has no new annotation yet".to_owned()),
            None => Err(format!(
                "Prosa's new annotation covers {} rather than {}",
                new.iter()
                    .map(|annotation| quoted(&covered(ctx, annotation)))
                    .collect::<Vec<_>>()
                    .join(", "),
                quoted(phrase)
            )),
        }
    });

    ctx.prompt.detail("From", &annotation.start_location);
    ctx.prompt.detail("To", &annotation.end_location);

    if annotation.note.as_deref() != note {
        return Err(fail(format!(
            "The annotation's note is {:?}, not {:?}",
            annotation.note, note
        )));
    }

    Ok(annotation)
}

async fn device_highlight(ctx: &mut Ctx) -> StepResult {
    let annotation = highlight_on_device(
        ctx,
        DEVICE_HIGHLIGHT,
        "by long-pressing its first word and dragging",
        None,
    )
    .await?;
    ctx.remember("device highlight", &annotation.annotation_id);

    Ok(())
}

async fn device_note(ctx: &mut Ctx) -> StepResult {
    let annotation = highlight_on_device(ctx, DEVICE_NOTE, "", Some(DEVICE_NOTE_TEXT)).await?;
    ctx.remember("device note", &annotation.annotation_id);

    Ok(())
}

async fn device_note_edited(ctx: &mut Ctx) -> StepResult {
    let annotation_id = ctx.recall("device note")?;
    let whereabouts = whereabouts(ctx.book(BOOK), DEVICE_NOTE);
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            &format!("Open the book at {whereabouts}"),
            &format!(
                "Tap the note on {}, edit it to read “{DEVICE_NOTE_EDITED}” and save",
                quoted(DEVICE_NOTE)
            ),
            CLOSE,
            SYNC,
        ],
    )?;

    let book_id = ctx.id(BOOK)?;
    until!(ctx, {
        let annotation = ctx
            .api()
            .get_annotation(&book_id, &annotation_id, ctx.key())
            .await?;
        if annotation.note.as_deref() == Some(DEVICE_NOTE_EDITED) {
            Ok(())
        } else {
            Err(format!("Prosa's note still reads {:?}", annotation.note))
        }
    });

    Ok(())
}

async fn device_deleted(ctx: &mut Ctx) -> StepResult {
    let annotation_id = ctx.recall("device highlight")?;
    let whereabouts = whereabouts(ctx.book(BOOK), DEVICE_HIGHLIGHT);
    ctx.prompt.instruct(
        "On the Kobo",
        &[
            &format!("Open the book at {whereabouts}"),
            &format!("Tap the highlight on {} and delete it", quoted(DEVICE_HIGHLIGHT)),
            CLOSE,
            SYNC,
        ],
    )?;

    let book_id = ctx.id(BOOK)?;
    until!(ctx, {
        let remaining = ctx.api().list_annotations(&book_id, ctx.key()).await?;
        if remaining.contains(&annotation_id) {
            Err("Prosa still has the highlight".to_owned())
        } else {
            Ok(())
        }
    });

    Ok(())
}

async fn device_one_character(ctx: &mut Ctx) -> StepResult {
    highlight_on_device(
        ctx,
        DEVICE_ONE_CHARACTER,
        "(long-press the lone letter; leave the commas out)",
        None,
    )
    .await
    .map(drop)
}

async fn device_paragraph_end(ctx: &mut Ctx) -> StepResult {
    highlight_on_device(
        ctx,
        DEVICE_PARAGRAPH_END,
        "(the paragraph's last words, which have no full stop)",
        None,
    )
    .await
    .map(drop)
}

async fn device_across_paragraphs(ctx: &mut Ctx) -> StepResult {
    highlight_on_device(
        ctx,
        DEVICE_ACROSS_PARAGRAPHS,
        "from “cormorants” at the end of [2.10] to “arrived” in [2.11]",
        None,
    )
    .await
    .map(drop)
}

async fn device_across_formatting(ctx: &mut Ctx) -> StepResult {
    highlight_on_device(ctx, DEVICE_ACROSS_FORMATTING, "(“very” is in italics)", None)
        .await
        .map(drop)
}

async fn device_accented(ctx: &mut Ctx) -> StepResult {
    highlight_on_device(ctx, DEVICE_ACCENTED, "", None)
        .await
        .map(drop)
}

async fn add_elsewhere(ctx: &Ctx, start: String, end: String, note: Option<&str>) -> Result<String, StepEnd> {
    let request = ProsaAnnotationRequest {
        start_location: start,
        end_location: end,
        note: note.map(ToOwned::to_owned),
        annotation_id: None,
    };

    Ok(ctx
        .api()
        .add_annotation(&ctx.id(BOOK)?, &request, ctx.key())
        .await?)
}

async fn highlight_elsewhere(ctx: &Ctx, phrase: &str, note: Option<&str>) -> Result<String, StepEnd> {
    let span = ctx.book(BOOK).locate(phrase);

    add_elsewhere(ctx, span.start, span.end, note).await
}

async fn expect_on_device(ctx: &Ctx, present: &[&str], absent: &[&str]) -> StepResult {
    let sent = device_annotation_ids(ctx, BOOK).await?;

    if let Some(missing) = present.iter().find(|id| !sent.iter().any(|sent| sent == *id)) {
        return Err(fail(format!(
            "prosa-kobo would not send the device annotation {missing}"
        )));
    }
    if let Some(leaked) = absent.iter().find(|id| sent.iter().any(|sent| sent == *id)) {
        return Err(fail(format!(
            "prosa-kobo would send the device annotation {leaked}"
        )));
    }

    Ok(())
}

fn open_at(ctx: &Ctx, phrase: &str) -> String {
    format!(
        "Open “The Lighthouse Keeper” at {}",
        whereabouts(ctx.book(BOOK), phrase)
    )
}

async fn prosa_added(ctx: &mut Ctx) -> StepResult {
    let highlight = highlight_elsewhere(ctx, PROSA_HIGHLIGHT, None).await?;
    let note = highlight_elsewhere(ctx, PROSA_NOTE, Some(PROSA_NOTE_TEXT)).await?;
    ctx.remember("prosa highlight", &highlight);
    ctx.remember("prosa note", &note);

    expect_on_device(ctx, &[&highlight, &note], &[]).await?;

    let open = open_at(ctx, PROSA_HIGHLIGHT);
    ctx.prompt.instruct("On the Kobo", &[SYNC, &open])?;

    ctx.prompt.confirm(&format!(
        "Is exactly {} highlighted in [5.5], and {} in [5.6] with the note “{PROSA_NOTE_TEXT}”?",
        quoted(PROSA_HIGHLIGHT),
        quoted(PROSA_NOTE)
    ))
}

async fn prosa_note_edited(ctx: &mut Ctx) -> StepResult {
    let annotation_id = ctx.recall("prosa note")?;
    ctx.api()
        .patch_annotation(&ctx.id(BOOK)?, &annotation_id, PROSA_NOTE_EDITED, ctx.key())
        .await?;

    let open = open_at(ctx, PROSA_NOTE);
    ctx.prompt.instruct("On the Kobo", &[CLOSE, SYNC, &open])?;

    ctx.prompt.confirm(&format!(
        "Does the note on {} now read “{PROSA_NOTE_EDITED}”?",
        quoted(PROSA_NOTE)
    ))
}

async fn prosa_deleted(ctx: &mut Ctx) -> StepResult {
    let annotation_id = ctx.recall("prosa highlight")?;
    ctx.api()
        .delete_annotation(&ctx.id(BOOK)?, &annotation_id, ctx.key())
        .await?;

    expect_on_device(ctx, &[], &[&annotation_id]).await?;

    let open = open_at(ctx, PROSA_HIGHLIGHT);
    ctx.prompt.instruct("On the Kobo", &[CLOSE, SYNC, &open])?;

    ctx.prompt.confirm(&format!(
        "Is {} no longer highlighted, while the note on {} remains?",
        quoted(PROSA_HIGHLIGHT),
        quoted(PROSA_NOTE)
    ))
}

async fn prosa_edge_cases(ctx: &mut Ctx) -> StepResult {
    let cases = [
        (
            PROSA_ONE_CHARACTER,
            "Is the lone letter Z in [5.7] highlighted, and nothing around it?",
        ),
        (
            PROSA_PARAGRAPH_END,
            "Is exactly “fog rolled in”, the last words of [5.8], highlighted?",
        ),
        (
            PROSA_ACROSS_PARAGRAPHS,
            "Is everything from “the pier.” ending [5.9] to “Thunder” in [5.10] highlighted, and no more?",
        ),
    ];

    let mut ids = Vec::new();
    for (phrase, _) in cases {
        ids.push(highlight_elsewhere(ctx, phrase, None).await?);
    }
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    expect_on_device(ctx, &ids, &[]).await?;

    let open = open_at(ctx, PROSA_ONE_CHARACTER);
    ctx.prompt.instruct("On the Kobo", &[CLOSE, SYNC, &open])?;

    let mut wrong = Vec::new();
    for (phrase, question) in cases {
        match ctx.prompt.confirm(question) {
            Ok(()) => {}
            Err(StepEnd::Failed(reason)) => wrong.push(format!("{}: {reason}", quoted(phrase))),
            Err(end) => return Err(end),
        }
    }

    if wrong.is_empty() {
        Ok(())
    } else {
        Err(fail(wrong.join("; ")))
    }
}

async fn prosa_untranslatable(ctx: &mut Ctx) -> StepResult {
    let start = ctx.book(BOOK).text_location(BLANK_CHAPTER, &[1], 0);
    let end = ctx.book(BOOK).text_location(BLANK_CHAPTER, &[1], 3);
    let untranslatable = add_elsewhere(ctx, start, end, None).await?;
    let beside = highlight_elsewhere(ctx, PROSA_BESIDE_UNTRANSLATABLE, None).await?;

    expect_on_device(ctx, &[&beside], &[&untranslatable]).await?;
    ctx.prompt.good(
        "prosa-kobo leaves out the highlight of bare whitespace, which the kepub has no span for, and keeps the other",
    );

    let open = open_at(ctx, PROSA_BESIDE_UNTRANSLATABLE);
    ctx.prompt.instruct("On the Kobo", &[CLOSE, SYNC, &open])?;

    ctx.prompt.confirm(&format!(
        "Did the book open without complaint, with {} highlighted in [5.12]?",
        quoted(PROSA_BESIDE_UNTRANSLATABLE)
    ))
}
