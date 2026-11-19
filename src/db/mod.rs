//! SQLite storage: opening, versioned migrations (`PRAGMA user_version`) and statistics.
//! Schema reference: `docs/05-DATA-SCHEMAS.md`.

mod import;

pub use import::{ImportRun, ImportSummary, import_dataset};

use std::path::Path;

use rusqlite::{Connection, OptionalExtension};

/// File name of the database inside the data directory.
pub const DATABASE_FILE: &str = "plop-warframe.sqlite3";

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("base de données : {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("version de base {found} plus récente que celle de l'application ({supported})")]
    TooNew { found: i64, supported: i64 },
}

/// Migrations in order; migration `n` (1-based) brings `user_version` from `n - 1` to `n`.
const MIGRATIONS: &[&str] = &[include_str!("migrations/001_initial.sql")];

fn latest_version() -> i64 {
    i64::try_from(MIGRATIONS.len()).unwrap_or(i64::MAX)
}

/// Opens (or creates) the database file and brings its schema up to date.
pub fn open(path: &Path) -> Result<Connection, DbError> {
    prepare(Connection::open(path)?)
}

/// In-memory database with the current schema, for tests.
pub fn open_in_memory() -> Result<Connection, DbError> {
    prepare(Connection::open_in_memory()?)
}

fn prepare(mut conn: Connection) -> Result<Connection, DbError> {
    conn.pragma_update(None, "foreign_keys", true)?;
    migrate(&mut conn)?;
    Ok(conn)
}

fn migrate(conn: &mut Connection) -> Result<(), DbError> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let latest = latest_version();
    if current > latest {
        return Err(DbError::TooNew {
            found: current,
            supported: latest,
        });
    }
    for (index, sql) in MIGRATIONS
        .iter()
        .enumerate()
        .skip(usize::try_from(current).unwrap_or(0))
    {
        let target = i64::try_from(index + 1).unwrap_or(i64::MAX);
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", target)?;
        tx.commit()?;
    }
    Ok(())
}

pub fn schema_version(conn: &Connection) -> Result<i64, DbError> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Last import attempt, successful or not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastRun {
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub source: String,
    pub source_revision: Option<String>,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    pub nodes: i64,
    pub items: i64,
    pub drops: i64,
    pub last_run: Option<LastRun>,
}

pub fn stats(conn: &Connection) -> Result<Stats, DbError> {
    let count = |sql: &str| conn.query_row(sql, [], |row| row.get::<_, i64>(0));
    let last_run = conn
        .query_row(
            "SELECT started_at, finished_at, source, source_revision, status, message
             FROM import_run ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                Ok(LastRun {
                    started_at: row.get(0)?,
                    finished_at: row.get(1)?,
                    source: row.get(2)?,
                    source_revision: row.get(3)?,
                    status: row.get(4)?,
                    message: row.get(5)?,
                })
            },
        )
        .optional()?;
    Ok(Stats {
        nodes: count("SELECT count(*) FROM node")?,
        items: count("SELECT count(*) FROM item")?,
        drops: count("SELECT count(*) FROM \"drop\"")?,
        last_run,
    })
}
