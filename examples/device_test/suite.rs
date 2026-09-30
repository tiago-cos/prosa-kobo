use crate::{
    account::Account,
    books::Fixture,
    epub::Book,
    middleware::Middleware,
    prompt::{Prompt, Tone},
    prosa::Prosa,
};
use prosa_kobo::client::{
    ProsaAnnotation,
    prosa::{ClientError, ProsaApi},
    state::ProsaState,
};
use std::{collections::HashMap, fmt::Write as _, future::Future, pin::Pin};

pub const SYNC: &str = "Tap the sync icon at the top of the Home screen and wait for it to finish";
pub const CLOSE: &str = "Go back to the Home screen, which closes the book and records where you are";

#[derive(Debug)]
pub struct Problem(pub String);

impl Problem {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<reqwest::Error> for Problem {
    fn from(error: reqwest::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<ClientError> for Problem {
    fn from(error: ClientError) -> Self {
        Self(format!("Prosa answered {error:?}"))
    }
}

pub enum StepEnd {
    Failed(String),
    Skipped(String),
    Quit,
}

impl From<Problem> for StepEnd {
    fn from(problem: Problem) -> Self {
        StepEnd::Failed(problem.0)
    }
}

impl From<ClientError> for StepEnd {
    fn from(error: ClientError) -> Self {
        Problem::from(error).into()
    }
}

pub type StepResult = Result<(), StepEnd>;
pub type StepFn = for<'a> fn(&'a mut Ctx) -> Pin<Box<dyn Future<Output = StepResult> + 'a>>;

pub struct Step {
    pub name: &'static str,
    pub run: StepFn,
}

pub struct Section {
    pub name: &'static str,
    pub summary: &'static str,
    pub steps: Vec<Step>,
}

macro_rules! steps {
    ($($name:literal => $step:path),* $(,)?) => {
        vec![$($crate::suite::Step { name: $name, run: |ctx| Box::pin($step(ctx)) }),*]
    };
}

/// Runs a check until it holds, asking the tester what to do each time it
/// does not. The check is written as an expression so it is evaluated anew.
macro_rules! until {
    ($ctx:expr, $check:expr) => {
        loop {
            match $check {
                Ok(value) => break value,
                Err(problem) => $ctx.prompt.retry(&problem)?,
            }
        }
    };
}

pub(crate) use {steps, until};

pub fn fail(message: impl Into<String>) -> StepEnd {
    StepEnd::Failed(message.into())
}

pub struct Ctx {
    pub prosa: Prosa,
    pub middleware: Middleware,
    pub account: Account,
    pub prompt: Prompt,
    books: HashMap<Fixture, String>,
    memo: HashMap<&'static str, String>,
    generated: HashMap<Fixture, Book>,
}

impl Ctx {
    pub fn new(prosa: Prosa, middleware: Middleware, account: Account, prompt: Prompt) -> Self {
        Self {
            prosa,
            middleware,
            account,
            prompt,
            books: HashMap::new(),
            memo: HashMap::new(),
            generated: [
                Fixture::Lighthouse,
                Fixture::Orchard,
                Fixture::OrchardRevised,
                Fixture::Tidepool,
                Fixture::Bare,
            ]
            .into_iter()
            .map(|fixture| (fixture, fixture.book()))
            .collect(),
        }
    }

    pub fn api(&self) -> &dyn ProsaApi {
        &self.prosa.api
    }

    pub fn key(&self) -> &str {
        &self.account.other_key
    }

    pub fn book(&self, fixture: Fixture) -> &Book {
        &self.generated[&fixture]
    }

    pub fn id(&self, fixture: Fixture) -> Result<String, StepEnd> {
        self.books
            .get(&fixture)
            .cloned()
            .ok_or_else(|| StepEnd::Skipped(format!("{} was never added", fixture.title())))
    }

    pub fn remember(&mut self, name: &'static str, value: &str) {
        self.memo.insert(name, value.to_owned());
    }

    pub fn recall(&self, name: &'static str) -> Result<String, StepEnd> {
        self.memo
            .get(name)
            .cloned()
            .ok_or_else(|| StepEnd::Skipped(format!("an earlier step did not record {name}")))
    }

    pub async fn empty_library(&mut self) -> Result<(), Problem> {
        let username = self.account.username.clone();
        let key = self.key().to_owned();

        for shelf_id in self.prosa.shelves_of(&username, &key).await? {
            self.prosa.api.delete_shelf(&shelf_id, &key).await?;
        }
        for book_id in self.prosa.books_of(&username, &key).await? {
            self.prosa.api.delete_book(&book_id, &key).await?;
        }

        self.books.clear();
        self.memo.clear();

        Ok(())
    }

    pub async fn add(&mut self, fixture: Fixture, book_id: Option<&str>) -> Result<String, Problem> {
        let key = self.key().to_owned();
        let book_id = self.prosa.stock(fixture, book_id, &key).await?;
        self.books.insert(fixture, book_id.clone());

        Ok(book_id)
    }

    pub fn forget(&mut self, fixture: Fixture) {
        self.books.remove(&fixture);
    }

    /// Every section starts from a library holding only what it needs, so a
    /// section can be run alone and no earlier section can leak into it.
    pub async fn library(&mut self, fixtures: &[Fixture]) -> StepResult {
        self.empty_library().await?;
        for fixture in fixtures {
            self.add(*fixture, None).await?;
        }

        let titles: Vec<String> = fixtures.iter().map(|f| format!("“{}”", f.title())).collect();
        self.prompt.instruct(
            "On the Kobo",
            &[
                SYNC,
                "Let it remove any books left over from earlier sections",
                &format!("Wait for these to appear in My Books: {}", titles.join(", ")),
            ],
        )?;

        self.prompt.confirm(&format!(
            "Are {} in My Books, with no test book left from an earlier section? Books of your own may be there too.",
            titles.join(", ")
        ))
    }

    pub async fn state(&self, fixture: Fixture) -> Result<ProsaState, StepEnd> {
        Ok(self.api().fetch_state(&self.id(fixture)?, self.key()).await?)
    }

    pub async fn replace_state(&self, fixture: Fixture, state: &ProsaState) -> StepResult {
        Ok(self
            .api()
            .replace_state(&self.id(fixture)?, state, self.key())
            .await?)
    }

    pub async fn annotations(&self, fixture: Fixture) -> Result<Vec<ProsaAnnotation>, StepEnd> {
        let book_id = self.id(fixture)?;
        let mut annotations = Vec::new();
        for annotation_id in self.api().list_annotations(&book_id, self.key()).await? {
            annotations.push(
                self.api()
                    .get_annotation(&book_id, &annotation_id, self.key())
                    .await?,
            );
        }

        Ok(annotations)
    }
}

pub enum Outcome {
    Passed,
    Failed(String),
    Skipped(String),
}

pub struct Record {
    pub section: &'static str,
    pub step: &'static str,
    pub outcome: Outcome,
    pub notes: Vec<String>,
}

pub async fn run(ctx: &mut Ctx, sections: &[Section]) -> Vec<Record> {
    let mut records = Vec::new();

    'sections: for section in sections {
        ctx.prompt
            .heading(&format!("{} — {}", section.name, section.summary));

        for (index, step) in section.steps.iter().enumerate() {
            ctx.prompt.step(
                &format!("{} {}/{}", section.name, index + 1, section.steps.len()),
                step.name,
            );

            let outcome = match (step.run)(ctx).await {
                Ok(()) => Outcome::Passed,
                Err(StepEnd::Failed(reason)) => Outcome::Failed(reason),
                Err(StepEnd::Skipped(reason)) => Outcome::Skipped(reason),
                Err(StepEnd::Quit) => break 'sections,
            };

            let prompt = &ctx.prompt;
            match &outcome {
                Outcome::Passed => println!("  {}", prompt.paint(Tone::Good, "✔ passed")),
                Outcome::Failed(reason) => {
                    println!("  {}", prompt.paint(Tone::Bad, &format!("✘ failed: {reason}")));
                }
                Outcome::Skipped(_) => println!("  {}", prompt.paint(Tone::Dim, "– skipped")),
            }

            records.push(Record {
                section: section.name,
                step: step.name,
                outcome,
                notes: ctx.prompt.take_notes(),
            });
        }
    }

    records
}

pub fn summarize(prompt: &Prompt, records: &[Record]) -> String {
    let count = |matches: fn(&Outcome) -> bool| records.iter().filter(|r| matches(&r.outcome)).count();
    let passed = count(|o| matches!(o, Outcome::Passed));
    let failed = count(|o| matches!(o, Outcome::Failed(_)));
    let skipped = count(|o| matches!(o, Outcome::Skipped(_)));

    let mut report = String::new();
    let mut plain = String::new();
    let mut section = "";

    for record in records {
        if record.section != section {
            section = record.section;
            let _ = writeln!(report, "\n{}", prompt.paint(Tone::Bold, section));
            let _ = writeln!(plain, "\n## {section}\n");
        }

        let (mark, tone, detail) = match &record.outcome {
            Outcome::Passed => ("✔", Tone::Good, None),
            Outcome::Failed(reason) => ("✘", Tone::Bad, Some(reason.as_str())),
            Outcome::Skipped(reason) => ("–", Tone::Dim, Some(reason.as_str()).filter(|r| !r.is_empty())),
        };
        let _ = writeln!(report, "  {} {}", prompt.paint(tone, mark), record.step);
        let _ = writeln!(plain, "- {mark} {}", record.step);

        for line in detail
            .into_iter()
            .map(ToOwned::to_owned)
            .chain(record.notes.iter().map(|n| format!("note: {n}")))
        {
            let _ = writeln!(report, "      {}", prompt.paint(Tone::Dim, &line));
            let _ = writeln!(plain, "    - {line}");
        }
    }

    let totals = format!("{passed} passed · {failed} failed · {skipped} skipped");
    println!("{report}\n{}", prompt.paint(Tone::Bold, &totals));

    format!("# Kobo device test\n{plain}\n{totals}\n")
}
