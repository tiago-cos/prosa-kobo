use prosa_kobo::database::{connect, init, latest_version, revert_to, status};
use sqlx::SqlitePool;
use std::path::PathBuf;

const LEGACY_SCHEMA: &str = r"
    CREATE TABLE IF NOT EXISTS linked_devices (
        device_id TEXT PRIMARY KEY NOT NULL,
        lookup_key TEXT NOT NULL UNIQUE,
        user_id TEXT NOT NULL,
        name TEXT NOT NULL,
        api_key TEXT NOT NULL,
        client_device_id TEXT UNIQUE
    );

    CREATE TABLE IF NOT EXISTS cover_versions (
        device_id TEXT NOT NULL,
        book_id TEXT NOT NULL,
        version INTEGER NOT NULL,
        PRIMARY KEY(device_id, book_id)
    );

    CREATE TABLE IF NOT EXISTS etags (
        book_id TEXT PRIMARY KEY NOT NULL,
        etag TEXT NOT NULL
    );
";

struct Scratch {
    directory: PathBuf,
    database: String,
}

impl Scratch {
    fn new() -> Self {
        let directory =
            std::env::temp_dir().join(format!("prosa-kobo-migrations-{:016x}", rand::random::<u64>()));
        std::fs::create_dir_all(&directory).expect("Failed to create a scratch directory");
        let database = directory.join("database.db").to_string_lossy().into_owned();

        Self { directory, database }
    }

    fn backups(&self, label: &str) -> usize {
        self.files_starting(&format!("database.db.backup-{label}-"))
    }

    fn database_backups(&self) -> usize {
        self.files_starting("database.db.backup-")
    }

    fn files_starting(&self, prefix: &str) -> usize {
        std::fs::read_dir(&self.directory)
            .expect("Failed to list the scratch directory")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
            .count()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

async fn tables(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar(
        r"
        SELECT name
        FROM sqlite_master
        WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations'
        ORDER BY name
        ",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to list tables")
}

async fn link_device(pool: &SqlitePool) {
    sqlx::query(
        r"
        INSERT INTO linked_devices (device_id, lookup_key, user_id, name, api_key, client_device_id)
        VALUES ('device', 'lookup', 'user', 'Kobo', 'key', 'N123')
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to link a device");
}

async fn linked_devices(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar("SELECT device_id FROM linked_devices")
        .fetch_all(pool)
        .await
        .expect("Failed to read linked devices")
}

fn schema() -> Vec<String> {
    vec![
        "cover_versions".to_owned(),
        "device_states".to_owned(),
        "etags".to_owned(),
        "linked_devices".to_owned(),
    ]
}

#[tokio::test]
async fn a_new_database_is_migrated_to_the_latest_version_without_a_backup() {
    let scratch = Scratch::new();

    let pool = init(&scratch.database).await.expect("Failed to initialize");

    assert_eq!(tables(&pool).await, schema());
    assert!(
        status(&pool)
            .await
            .expect("Failed to report status")
            .starts_with(&format!("Schema version: {}\n", latest_version()))
    );
    assert_eq!(scratch.database_backups(), 0);
}

#[tokio::test]
async fn reports_the_migrations_a_first_start_will_run() {
    let scratch = Scratch::new();

    let pool = connect(&scratch.database).await.expect("Failed to connect");
    let report = status(&pool).await.expect("Failed to report status");

    assert!(report.starts_with("Schema version: 0\n"));
    assert!(report.contains("pending"));
    assert!(report.ends_with("migration(s) will run on next start.\n"));
}

#[tokio::test]
async fn adopts_a_database_made_before_migrations_keeping_its_rows() {
    let scratch = Scratch::new();
    let legacy = connect(&scratch.database).await.expect("Failed to connect");
    sqlx::raw_sql(LEGACY_SCHEMA)
        .execute(&legacy)
        .await
        .expect("Failed to create the legacy schema");
    link_device(&legacy).await;
    legacy.close().await;

    let pool = init(&scratch.database).await.expect("Failed to initialize");

    assert_eq!(linked_devices(&pool).await, vec!["device".to_owned()]);
    assert!(
        !status(&pool)
            .await
            .expect("Failed to report status")
            .contains("pending")
    );
    assert_eq!(scratch.backups("pre-upgrade-v0"), 1);
}

#[tokio::test]
async fn starting_again_changes_nothing() {
    let scratch = Scratch::new();
    init(&scratch.database)
        .await
        .expect("Failed to initialize")
        .close()
        .await;

    let pool = init(&scratch.database).await.expect("Failed to initialize again");

    assert_eq!(tables(&pool).await, schema());
    assert_eq!(scratch.database_backups(), 0);
}

#[tokio::test]
async fn reverting_to_zero_backs_up_and_drops_the_schema_which_the_next_start_restores() {
    let scratch = Scratch::new();
    let pool = init(&scratch.database).await.expect("Failed to initialize");
    link_device(&pool).await;

    revert_to(&pool, 0, &scratch.database)
        .await
        .expect("Failed to revert");

    assert!(tables(&pool).await.is_empty());
    assert_eq!(scratch.backups("pre-downgrade-to-v0"), 1);
    pool.close().await;

    let pool = init(&scratch.database).await.expect("Failed to initialize again");
    assert_eq!(tables(&pool).await, schema());
}

#[tokio::test]
async fn reverting_to_the_first_version_forgets_what_devices_hold_but_keeps_the_devices() {
    let scratch = Scratch::new();
    let pool = init(&scratch.database).await.expect("Failed to initialize");
    link_device(&pool).await;

    revert_to(&pool, 1, &scratch.database)
        .await
        .expect("Failed to revert");

    assert!(!tables(&pool).await.contains(&"device_states".to_owned()));
    assert_eq!(linked_devices(&pool).await, vec!["device".to_owned()]);
}

#[tokio::test]
async fn refuses_a_negative_target() {
    let scratch = Scratch::new();
    let pool = init(&scratch.database).await.expect("Failed to initialize");

    assert!(revert_to(&pool, -1, &scratch.database).await.is_err());
    assert_eq!(tables(&pool).await, schema());
}
