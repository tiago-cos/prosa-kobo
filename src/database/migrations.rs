use super::connection::DatabaseError;
use crate::CONFIG;
use chrono::Utc;
use log::{info, warn};
use sqlx::{SqlitePool, migrate::Migrator};
use std::{fmt::Write, path::Path};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub fn latest_version() -> i64 {
    MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .map(|migration| migration.version)
        .max()
        .unwrap_or_default()
}

async fn applied_versions(pool: &SqlitePool) -> Result<Vec<i64>, sqlx::Error> {
    let table: Option<String> = sqlx::query_scalar(
        r"
        SELECT name
        FROM sqlite_master
        WHERE type = 'table' AND name = '_sqlx_migrations'
        ",
    )
    .fetch_optional(pool)
    .await?;

    if table.is_none() {
        return Ok(Vec::new());
    }

    sqlx::query_scalar(
        r"
        SELECT version
        FROM _sqlx_migrations
        ORDER BY version
        ",
    )
    .fetch_all(pool)
    .await
}

async fn pending_versions(pool: &SqlitePool) -> Result<Vec<i64>, sqlx::Error> {
    let applied = applied_versions(pool).await?;

    let pending = MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .map(|migration| migration.version)
        .filter(|version| !applied.contains(version))
        .collect();

    Ok(pending)
}

async fn is_populated(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let tables: i64 = sqlx::query_scalar(
        r"
        SELECT COUNT(*)
        FROM sqlite_master
        WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
        ",
    )
    .fetch_one(pool)
    .await?;

    Ok(tables > 0)
}

async fn backup(pool: &SqlitePool, filename: &str, label: &str) -> Result<String, DatabaseError> {
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let target = format!("{filename}.backup-{label}-{timestamp}");

    if Path::new(&target).exists() {
        return Err(format!("backup target '{target}' already exists").into());
    }

    sqlx::query("VACUUM INTO ?").bind(&target).execute(pool).await?;

    Ok(target)
}

async fn backup_if_enabled(pool: &SqlitePool, filename: &str, label: &str) -> Result<(), DatabaseError> {
    if !CONFIG.database.backup_before_migration {
        warn!("Skipping pre-migration backup: database.backup_before_migration is disabled");
        return Ok(());
    }

    if !is_populated(pool).await? {
        return Ok(());
    }

    let target = backup(pool, filename, label).await?;
    info!("Backed up database to '{target}'");

    Ok(())
}

pub async fn apply(pool: &SqlitePool, filename: &str) -> Result<(), DatabaseError> {
    let pending = pending_versions(pool).await?;

    if pending.is_empty() {
        info!("Database schema is up to date at version {}", latest_version());
        return Ok(());
    }

    let current = applied_versions(pool).await?.last().copied().unwrap_or_default();
    backup_if_enabled(pool, filename, &format!("pre-upgrade-v{current}")).await?;

    info!("Applying {} pending migration(s): {pending:?}", pending.len());
    MIGRATOR.run(pool).await?;
    info!("Database schema migrated to version {}", latest_version());

    Ok(())
}

pub async fn revert_to(pool: &SqlitePool, target: i64, filename: &str) -> Result<(), DatabaseError> {
    if target < 0 {
        return Err("target version must not be negative".into());
    }

    let applied = applied_versions(pool).await?;
    let to_revert: Vec<i64> = applied.into_iter().filter(|version| *version > target).collect();

    if to_revert.is_empty() {
        info!("Nothing to revert: schema is already at or below version {target}");
        return Ok(());
    }

    for version in &to_revert {
        let revertible = MIGRATOR
            .iter()
            .any(|migration| migration.version == *version && migration.migration_type.is_down_migration());

        if !revertible {
            return Err(format!(
                "this binary carries no down migration for version {version}; \
                 run --migrate-down using the prosa-kobo version that introduced it, \
                 then downgrade"
            )
            .into());
        }
    }

    backup_if_enabled(pool, filename, &format!("pre-downgrade-to-v{target}")).await?;

    warn!("Reverting migration(s) {to_revert:?}; schema will drop to version {target}");
    MIGRATOR.undo(pool, target).await?;
    info!("Database schema reverted to version {target}");

    Ok(())
}

pub async fn status(pool: &SqlitePool) -> Result<String, DatabaseError> {
    let applied = applied_versions(pool).await?;
    let pending = pending_versions(pool).await?;

    let mut report = String::new();
    writeln!(
        report,
        "Schema version: {}",
        applied.last().copied().unwrap_or_default()
    )?;
    writeln!(report, "Latest available: {}", latest_version())?;
    writeln!(report)?;

    for migration in MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
    {
        let state = if applied.contains(&migration.version) {
            "applied"
        } else {
            "pending"
        };

        let reversible = if migration.migration_type.is_reversible() {
            "reversible"
        } else {
            "irreversible"
        };

        writeln!(
            report,
            "  {:>4}  {state:<8} {reversible:<12} {}",
            migration.version, migration.description
        )?;
    }

    if !pending.is_empty() {
        writeln!(report)?;
        writeln!(report, "{} migration(s) will run on next start.", pending.len())?;
    }

    Ok(report)
}
