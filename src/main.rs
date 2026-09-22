use prosa_kobo::{CONFIG, app, database};
use std::{io::Error, path::Path};
use tokio::fs;

#[tokio::main]
async fn main() {
    create_parent_dir(&CONFIG.database.file_path).await.unwrap();

    let db_pool = database::init(&CONFIG.database.file_path).await;

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

    app::run(db_pool).await;
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
