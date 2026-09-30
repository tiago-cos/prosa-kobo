//! A semi-automated test of prosa-kobo against a real Kobo.
//!
//! The suite drives Prosa as another device would and tells the tester what
//! to do on the Kobo, then checks each change arrived on the other side. It
//! needs a running Prosa and a running prosa-kobo. See the README.

mod account;
mod books;
mod epub;
mod middleware;
mod prompt;
mod prosa;
mod sections;
mod suite;

use account::Account;
use middleware::Middleware;
use prompt::{Prompt, Tone};
use prosa::Prosa;
use suite::{Ctx, Problem, Section, StepEnd};

const USAGE: &str = "\
Usage: cargo run --example device_test -- [options]

  --only <sections>       run only these sections, comma separated
  --list                  list the sections and their steps
  --show-config           print the eReader.conf lines for the linked Kobo and stop
  --kobo-address <addr>   host:port the Kobo reaches prosa-kobo at
  --prosa <url>           Prosa's address        [$PROSA_URL, http://127.0.0.1:5000]
  --middleware <url>      prosa-kobo's address   [$MIDDLEWARE_URL, http://127.0.0.1:5001]
  --admin-key <key>       Prosa's admin key, if registration needs it [$PROSA_ADMIN_KEY]
  --reset                 empty the test library, unlink the Kobo and forget the account
";

#[derive(Default)]
struct Options {
    only: Vec<String>,
    list: bool,
    show_config: bool,
    reset: bool,
    kobo_address: Option<String>,
    prosa: Option<String>,
    middleware: Option<String>,
    admin_key: Option<String>,
}

fn options() -> Result<Options, String> {
    let mut options = Options::default();
    let mut arguments = std::env::args().skip(1);

    while let Some(argument) = arguments.next() {
        let mut value = || arguments.next().ok_or(format!("{argument} needs a value"));
        match argument.as_str() {
            "--only" => options
                .only
                .extend(value()?.split(',').map(|s| s.trim().to_owned())),
            "--list" => options.list = true,
            "--show-config" => options.show_config = true,
            "--reset" => options.reset = true,
            "--kobo-address" => options.kobo_address = Some(value()?),
            "--prosa" => options.prosa = Some(value()?),
            "--middleware" => options.middleware = Some(value()?),
            "--admin-key" => options.admin_key = Some(value()?),
            "-h" | "--help" => return Err(String::new()),
            _ => return Err(format!("Unknown option {argument}")),
        }
    }

    Ok(options)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let options = match options() {
        Ok(options) => options,
        Err(problem) => {
            if !problem.is_empty() {
                eprintln!("{problem}\n");
            }
            eprint!("{USAGE}");
            std::process::exit(2);
        }
    };

    let sections = match chosen(sections::all(), &options.only) {
        Ok(sections) => sections,
        Err(problem) => exit(&problem),
    };

    if options.list {
        for section in &sections {
            println!("{} — {}", section.name, section.summary);
            for step in &section.steps {
                println!("    {}", step.name);
            }
        }
        return;
    }

    if let Err(problem) = start(options, sections).await {
        exit(&problem.0);
    }
}

fn exit(problem: &str) -> ! {
    eprintln!("{problem}");
    std::process::exit(1);
}

fn chosen(all: Vec<Section>, only: &[String]) -> Result<Vec<Section>, String> {
    if only.is_empty() {
        return Ok(all);
    }

    let names: Vec<&str> = all.iter().map(|section| section.name).collect();
    if let Some(unknown) = only.iter().find(|name| !names.contains(&name.as_str())) {
        return Err(format!(
            "There is no section {unknown}. Sections: {}",
            names.join(", ")
        ));
    }

    Ok(all
        .into_iter()
        .filter(|section| only.iter().any(|name| name == section.name))
        .collect())
}

fn setting(given: Option<String>, variable: &str, default: &str) -> String {
    given
        .or_else(|| std::env::var(variable).ok())
        .unwrap_or_else(|| default.to_owned())
}

async fn start(options: Options, sections: Vec<Section>) -> Result<(), Problem> {
    let prosa_url = setting(options.prosa, "PROSA_URL", "http://127.0.0.1:5000");
    let middleware_url = setting(options.middleware, "MIDDLEWARE_URL", "http://127.0.0.1:5001");
    let admin_key = options
        .admin_key
        .or_else(|| std::env::var("PROSA_ADMIN_KEY").ok());

    let prosa = Prosa::new(&prosa_url, admin_key)?;
    prosa.health().await?;
    let middleware = Middleware::new(&middleware_url);
    let mut prompt = Prompt::new();

    let account = match Account::load()? {
        Some(account) if account.prosa_url != prosa_url => {
            return Err(Problem::new(format!(
                "The test account belongs to the Prosa at {}. Run with --reset to start over against {prosa_url}.",
                account.prosa_url
            )));
        }
        Some(account) => account,
        None if options.reset => return Err(Problem::new("There is no test account to reset.")),
        None => create_account(
            &prosa,
            &middleware,
            &mut prompt,
            &prosa_url,
            options.kobo_address.as_deref(),
        )
        .await
        .map_err(stopped)?,
    };

    let session = prosa.login(&account.username, &account.password).await;

    if options.reset {
        return reset(prosa, middleware, account, prompt, session.ok()).await;
    }

    let session = session.map_err(|e| {
        Problem::new(format!(
            "The test account cannot log in to Prosa ({e}). Run with --reset."
        ))
    })?;
    let mut ctx = Ctx::new(prosa, middleware, account, prompt);

    if !ctx
        .middleware
        .is_linked(&session.jwt_token, &ctx.account.user_id, &ctx.account.device_id)
        .await?
    {
        ctx.prompt
            .warn("prosa-kobo no longer knows the test Kobo, so it is linked again.");
        let address = match options.kobo_address {
            Some(address) => address,
            None => ask_kobo_address(&ctx.prompt).map_err(stopped)?,
        };
        let link = ctx
            .middleware
            .link(&session.jwt_token, &ctx.account.device_key, &address)
            .await?;
        ctx.account.device_id = link.device_id;
        ctx.account.api_endpoint = link.api_endpoint;
        ctx.account.save()?;
        show_config(&mut ctx.prompt, &ctx.account).map_err(stopped)?;
    } else if options.show_config {
        print_config(&ctx.account);
        return Ok(());
    }

    Prompt::say(&format!(
        "Testing as {} on {prosa_url} through {middleware_url}.\n\
         Each section replaces the test library with the books it needs.",
        ctx.account.username
    ));

    let records = suite::run(&mut ctx, &sections).await;
    let report = suite::summarize(&ctx.prompt, &records);
    let path = Account::directory().join("report.md");
    if std::fs::write(&path, report).is_ok() {
        println!(
            "{}",
            ctx.prompt
                .paint(Tone::Dim, &format!("Saved to {}", path.display()))
        );
    }

    Ok(())
}

/// Forgets the account even when Prosa or prosa-kobo no longer know it, so a
/// broken account can always be started over.
async fn reset(
    prosa: Prosa,
    middleware: Middleware,
    account: Account,
    prompt: Prompt,
    session: Option<prosa::Session>,
) -> Result<(), Problem> {
    let username = account.username.clone();
    let mut ctx = Ctx::new(prosa, middleware, account, prompt);

    match session {
        Some(session) => {
            ctx.empty_library().await?;
            if let Err(problem) = ctx
                .middleware
                .unlink(&session.jwt_token, &ctx.account.device_id)
                .await
            {
                ctx.prompt.warn(&format!("Could not unlink the Kobo: {problem}"));
            }
        }
        None => ctx.prompt.warn(&format!(
            "{username} cannot log in to Prosa, so its library is left as it is."
        )),
    }

    Account::forget()?;
    println!("Forgot the test account {username}. The next run creates a new one.");

    Ok(())
}

fn stopped(end: StepEnd) -> Problem {
    match end {
        StepEnd::Failed(reason) | StepEnd::Skipped(reason) => Problem::new(reason),
        StepEnd::Quit => Problem::new("Stopped."),
    }
}

fn ask_kobo_address(prompt: &Prompt) -> Result<String, StepEnd> {
    prompt.ask(
        "Which host:port does the Kobo reach prosa-kobo at? (for example 192.168.1.20:5001)\n  \
         Behind a reverse proxy, give the proxy's address and set server.public instead.",
    )
}

async fn create_account(
    prosa: &Prosa,
    middleware: &Middleware,
    prompt: &mut Prompt,
    prosa_url: &str,
    kobo_address: Option<&str>,
) -> Result<Account, StepEnd> {
    prompt.heading("First run");
    Prompt::say("Creating a Prosa user of its own for the suite, so your own library is never touched.");

    let suffix = format!("{:08x}", rand::random::<u32>());
    let username = format!("kobo-test-{suffix}");
    let password = format!("device-test-{:016x}", rand::random::<u64>());

    let session = prosa
        .register(&username, &password)
        .await
        .map_err(|e| StepEnd::Failed(format!("Could not register {username}: {e}")))?;
    let device_key = prosa.create_key(&session, "Kobo under test").await?;
    let other_key = prosa.create_key(&session, "Device test suite").await?;
    prosa.disable_automatic_metadata(&session).await?;

    let address = match kobo_address {
        Some(address) => address.to_owned(),
        None => ask_kobo_address(prompt)?,
    };
    let link = middleware.link(&session.jwt_token, &device_key, &address).await?;

    let account = Account {
        prosa_url: prosa_url.to_owned(),
        username,
        password,
        user_id: session.user_id,
        device_key,
        other_key,
        device_id: link.device_id,
        api_endpoint: link.api_endpoint,
    };
    account.save()?;
    prompt.good(&format!(
        "Saved the account to {}",
        Account::directory().display()
    ));
    show_config(prompt, &account)?;

    Ok(account)
}

fn show_config(prompt: &mut Prompt, account: &Account) -> Result<(), StepEnd> {
    print_config(account);

    prompt.instruct(
        "On the Kobo",
        &[
            "Keep a copy of the lines you replace, to point the Kobo back at your own library later",
            "Edit eReader.conf as above and eject the Kobo",
            "Sign in if the Kobo asks, then sync once",
        ],
    )
}

fn print_config(account: &Account) {
    Prompt::say(&format!(
        "\nPoint the Kobo at the test account. In .kobo/Kobo/Kobo eReader.conf on the device,\n\
         under [OneStoreServices], set:\n\n    \
         api_endpoint={}\n",
        account.api_endpoint
    ));
}
