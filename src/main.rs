use prosa_kobo::{
    CONFIG, app,
    client::{self, prosa::Client},
    database,
};
use std::sync::Arc;
use std::{io::Error, path::Path};
use tokio::fs;

type StartupError = Box<dyn std::error::Error + Send + Sync>;

const USAGE: &str = "\
Prosa-Kobo - a middleware between Kobo devices and Prosa

Usage:
  prosa-kobo                        Run the server, applying any pending migrations
  prosa-kobo --migrate-status       Show applied and pending schema migrations
  prosa-kobo --migrate-down <ver>   Revert the schema down to version <ver>
  prosa-kobo --help                 Show this message

Downgrading:
  A binary can only revert migrations whose down scripts it carries, so run
  --migrate-down with the newer prosa-kobo build *before* swapping in the older one.
";

enum Command {
    Serve,
    MigrateStatus,
    MigrateDown(i64),
}

#[tokio::main]
async fn main() {
    if let Err(error) = start().await {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

async fn start() -> Result<(), StartupError> {
    let Some(command) = parse_args()? else {
        print!("{USAGE}");
        return Ok(());
    };

    if matches!(command, Command::Serve) {
        print_banner();
    }

    app::init_logging();
    create_parent_dir(&CONFIG.database.file_path).await?;

    match command {
        Command::MigrateStatus => {
            let pool = database::connect(&CONFIG.database.file_path).await?;
            let report = database::status(&pool).await?;
            pool.close().await;
            print!("{report}");
        }
        Command::MigrateDown(target) => {
            let pool = database::connect(&CONFIG.database.file_path).await?;
            let result = database::revert_to(&pool, target, &CONFIG.database.file_path).await;
            pool.close().await;
            result?;
        }
        Command::Serve => {
            let pool = database::init(&CONFIG.database.file_path).await?;
            database::set_pool(pool)?;
            client::set_prosa_client(Arc::new(Client::new(
                &CONFIG.prosa.scheme,
                &CONFIG.prosa.host,
                CONFIG.prosa.port,
            )))?;
            app::run().await;
            database::pool().close().await;
        }
    }

    Ok(())
}

fn parse_args() -> Result<Option<Command>, StartupError> {
    let mut args = std::env::args().skip(1);

    let command = match args.next().as_deref() {
        None => Command::Serve,
        Some("--help" | "-h") => return Ok(None),
        Some("--migrate-status") => Command::MigrateStatus,
        Some("--migrate-down") => {
            let target = args
                .next()
                .ok_or("--migrate-down requires a target schema version")?;

            let target = target
                .parse()
                .map_err(|_| format!("invalid target schema version '{target}'"))?;

            Command::MigrateDown(target)
        }
        Some(unknown) => return Err(format!("unknown argument '{unknown}'\n\n{USAGE}").into()),
    };

    if let Some(extra) = args.next() {
        return Err(format!("unexpected argument '{extra}'\n\n{USAGE}").into());
    }

    Ok(Some(command))
}

async fn create_parent_dir(path: &str) -> Result<(), Error> {
    let path = Path::new(path);

    if !path.exists()
        && let Some(parent) = path.parent()
    {
        fs::create_dir_all(parent).await?;
    }

    Ok(())
}

fn print_banner() {
    println!(
        r"
 ───────────────────────────────────────────────────────
  ____                            _  __     _           
 |  _ \ _ __ ___  ___  __ _      | |/ /___ | |__   ___  
 | |_) | '__/ _ \/ __|/ _` | ___ | ' // _ \| '_ \ / _ \ 
 |  __/| | | (_) \__ \ (_| | ___ | . \ (_) | |_) | (_) |
 |_|   |_|  \___/|___/\__,_|     |_|\_\___/|_.__/ \___/ 

 ───────────────────────────────────────────────────────
        "
    );
}
