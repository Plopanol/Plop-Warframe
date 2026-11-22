//! SQLite storage: opening, versioned migrations (`PRAGMA user_version`) and statistics.
//! Schema reference: `docs/05-DATA-SCHEMAS.md`.

mod import;
mod recipes;
mod relics;
mod search;
mod spot;
mod star_chart;
mod wfcd;

pub use import::{ImportRun, ImportSummary, import_dataset};
pub use recipes::{
    RecipeSpot, profile_level, recipe_spots, recipes, save_local_recipe, save_profile_level,
};
pub use relics::{RelicRow, WARFRAME_PARTS, relics_for_warframe, warframes};
pub use search::{
    CombinedRow, ItemDropRow, ItemHit, Location, MissionTypeRow, NodeFilter, RotationFilter,
    SearchFilter, fold_text, item_categories, mission_types, search_by_item, search_combined,
    search_items,
};
pub use spot::{
    EnemyDropRow, PlanetRow, SpotFilter, SpotRow, enemies_for_items, planets, spots_for_items,
};
pub use star_chart::{
    EXCLUDED_MISSION_TYPES, StarChartChoices, StarChartFilter, StarChartNode, star_chart_choices,
    star_chart_nodes,
};
pub use wfcd::{WfcdItemHit, search_wfcd_items, wfcd_categories, wfcd_item_detail};

use std::path::Path;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, OptionalExtension};

/// File name of the database inside the data directory.
pub const DATABASE_FILE: &str = "plop-warframe.sqlite3";

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("recette enregistrée illisible : {0}")]
    RecipeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Recipe(#[from] crate::recipes::RecipeError),
    #[error("base de données : {}", sqlite_reason(.0))]
    Sqlite(#[from] rusqlite::Error),
    #[error(
        "version de base {found} plus récente que celle de l'application ({supported}) : mettez à jour l'application"
    )]
    TooNew { found: i64, supported: i64 },
}

/// French explanation of the SQLite failures a user can act on; others keep SQLite's text.
fn sqlite_reason(error: &rusqlite::Error) -> String {
    use rusqlite::ErrorCode;
    let code = match error {
        rusqlite::Error::SqliteFailure(failure, _) => failure.code,
        other => return other.to_string(),
    };
    match code {
        ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => {
            "fichier occupé par un autre programme, fermez-le puis réessayez".to_owned()
        }
        ErrorCode::ReadOnly | ErrorCode::PermissionDenied => {
            "écriture refusée, vérifiez les droits sur le dossier de données".to_owned()
        }
        ErrorCode::CannotOpen => {
            "ouverture du fichier impossible, vérifiez le dossier de données".to_owned()
        }
        ErrorCode::DiskFull => "disque plein".to_owned(),
        ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase => {
            "fichier endommagé ou d'un autre format ; supprimez-le puis réimportez".to_owned()
        }
        _ => error.to_string(),
    }
}

/// Migrations in order; migration `n` (1-based) brings `user_version` from `n - 1` to `n`.
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/001_initial.sql"),
    include_str!("migrations/002_enemies.sql"),
    include_str!("migrations/003_relics.sql"),
    include_str!("migrations/004_relic_vaulted.sql"),
    include_str!("migrations/005_wfcd_items.sql"),
    include_str!("migrations/006_recipes.sql"),
];

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
    // Case- and accent-insensitive search (`search_items`); SQLite has no such collation.
    conn.create_scalar_function(
        "fold",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            Ok(ctx
                .get::<Option<String>>(0)?
                .map(|text| search::fold_text(&text)))
        },
    )?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreign_file_gets_a_readable_error() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join(DATABASE_FILE);
        std::fs::write(
            &path,
            "pas une base SQLite, mais un texte assez long pour un en-tête",
        )?;
        let message = open(&path).err().map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some(
                "base de données : fichier endommagé ou d'un autre format ; supprimez-le puis \
                 réimportez"
            )
        );
        Ok(())
    }

    #[test]
    fn busy_database_is_explained() {
        let error = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            None,
        );
        assert!(DbError::from(error).to_string().contains("autre programme"));
    }
}
