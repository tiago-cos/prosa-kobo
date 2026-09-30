use crate::suite::StepEnd;
use std::io::{BufRead, IsTerminal, Write};

pub struct Prompt {
    color: bool,
    notes: Vec<String>,
}

#[derive(Clone, Copy)]
pub enum Tone {
    Bold,
    Dim,
    Good,
    Bad,
    Warn,
    Accent,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            color: std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
            notes: Vec::new(),
        }
    }

    pub fn paint(&self, tone: Tone, content: &str) -> String {
        if !self.color {
            return content.to_owned();
        }
        let code = match tone {
            Tone::Bold => "1",
            Tone::Dim => "2",
            Tone::Good => "32",
            Tone::Bad => "31",
            Tone::Warn => "33",
            Tone::Accent => "36",
        };

        format!("\x1b[{code}m{content}\x1b[0m")
    }

    pub fn heading(&self, content: &str) {
        println!("\n{}", self.paint(Tone::Bold, &format!("━━ {content} ━━")));
    }

    pub fn step(&self, position: &str, name: &str) {
        println!(
            "\n{} {}",
            self.paint(Tone::Accent, &format!("▸ {position}")),
            self.paint(Tone::Bold, name)
        );
    }

    pub fn say(content: &str) {
        for line in content.lines() {
            println!("  {line}");
        }
    }

    pub fn detail(&self, label: &str, value: &str) {
        println!("  {} {value}", self.paint(Tone::Dim, &format!("{label}:")));
    }

    pub fn good(&self, content: &str) {
        println!("  {}", self.paint(Tone::Good, &format!("✔ {content}")));
    }

    pub fn warn(&self, content: &str) {
        println!("  {}", self.paint(Tone::Warn, &format!("! {content}")));
    }

    pub fn take_notes(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notes)
    }

    fn read(&self, hint: &str) -> Result<String, StepEnd> {
        print!("  {} ", self.paint(Tone::Dim, &format!("{hint} ›")));
        std::io::stdout().flush().ok();

        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => Err(StepEnd::Quit),
            Ok(_) => Ok(line.trim().to_owned()),
        }
    }

    /// Splits `n the cover stayed blue` into its command and the note after it.
    fn command(line: &str) -> (String, Option<String>) {
        let (command, note) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let note = note.trim();

        (
            command.to_lowercase(),
            (!note.is_empty()).then(|| note.to_owned()),
        )
    }

    pub fn ask(&self, question: &str) -> Result<String, StepEnd> {
        println!("  {question}");
        loop {
            let answer = self.read("")?;
            if !answer.is_empty() {
                return Ok(answer);
            }
        }
    }

    /// Anything typed besides a command is kept as a note on the step.
    pub fn instruct(&mut self, on: &str, lines: &[&str]) -> Result<(), StepEnd> {
        println!("  {}", self.paint(Tone::Warn, &format!("{on}:")));
        for (index, line) in lines.iter().enumerate() {
            println!("    {}. {line}", index + 1);
        }

        let line = self.read("Enter when done · s skip · q quit")?;
        let (command, note) = Self::command(&line);
        match command.as_str() {
            "s" | "skip" => Err(StepEnd::Skipped(note.unwrap_or_default())),
            "q" | "quit" => Err(StepEnd::Quit),
            "" | "y" | "yes" | "done" => Ok(()),
            _ => {
                self.notes.push(line);
                Ok(())
            }
        }
    }

    pub fn confirm(&mut self, question: &str) -> Result<(), StepEnd> {
        println!("  {}", self.paint(Tone::Warn, question));
        loop {
            let line = self.read("y yes · n no · s skip · q quit (a note may follow)")?;
            let (command, note) = Self::command(&line);
            match command.as_str() {
                "y" | "yes" => {
                    self.notes.extend(note);
                    return Ok(());
                }
                "n" | "no" => {
                    let note = match note {
                        Some(note) => note,
                        None => self.read("What did you see instead?")?,
                    };
                    return Err(StepEnd::Failed(format!("Tester: {note}")));
                }
                "s" | "skip" => return Err(StepEnd::Skipped(note.unwrap_or_default())),
                "q" | "quit" => return Err(StepEnd::Quit),
                _ => {}
            }
        }
    }

    /// A check that fails is usually the device not having synced yet, so
    /// the default is to look again.
    pub fn retry(&mut self, problem: &str) -> Result<(), StepEnd> {
        println!("  {}", self.paint(Tone::Bad, &format!("✘ {problem}")));
        let line = self.read("Enter check again · f fail · s skip · q quit")?;
        let (command, note) = Self::command(&line);
        match command.as_str() {
            "f" | "fail" => Err(StepEnd::Failed(match note {
                Some(note) => format!("{problem} (tester: {note})"),
                None => problem.to_owned(),
            })),
            "s" | "skip" => Err(StepEnd::Skipped(note.unwrap_or_default())),
            "q" | "quit" => Err(StepEnd::Quit),
            _ => Ok(()),
        }
    }
}
