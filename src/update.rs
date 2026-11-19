//! Import use case shared by the CLI and the UI: download (or read the cache), parse, store.
//! Blocking: run it on a worker thread.

use std::path::{Path, PathBuf};

use crate::db::{self, DbError, ImportRun, ImportSummary};
use crate::source::{
    CacheError, CachedModule, DropSource, FetchError, ModuleCache, ModuleFetcher, SourceError,
    WikiSource,
};

/// Wiki page and cache file of each module, in download order.
pub const MODULES: [(&str, &str); 2] = [
    ("Module:DropTables/data", "droptables_data.lua"),
    ("Module:Missions/data", "missions_data.lua"),
];

pub const CACHE_DIR: &str = "cache";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Download the modules, refresh the cache, then import.
    Wiki,
    /// Import the cached modules without network.
    Cache,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error(transparent)]
    Cache(#[from] CacheError),
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error(transparent)]
    Db(#[from] DbError),
    #[error("dossier de données « {path} » : {source}")]
    DataDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Import progress: `done` steps out of `total`, with a French status line for the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress<'a> {
    pub done: usize,
    pub total: usize,
    pub message: &'a str,
}

impl Progress<'_> {
    /// Completed share, from 0 to 1, for a progress bar.
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        let done = u16::try_from(self.done.min(self.total)).unwrap_or(u16::MAX);
        let total = u16::try_from(self.total).unwrap_or(u16::MAX);
        f32::from(done) / f32::from(total)
    }
}

/// Number of progress steps of an import: one per download, reading, writing.
pub fn step_count(origin: Origin) -> usize {
    match origin {
        Origin::Wiki => MODULES.len() + 2,
        Origin::Cache => 2,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateReport {
    pub summary: ImportSummary,
    pub warnings: Vec<String>,
}

/// Paths derived from the data directory (`docs/05-DATA-SCHEMAS.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPaths {
    pub dir: PathBuf,
    pub database: PathBuf,
    pub cache: PathBuf,
}

impl DataPaths {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        Self {
            database: dir.join(db::DATABASE_FILE),
            cache: dir.join(CACHE_DIR),
            dir,
        }
    }

    /// Default location: `data/` next to the executable in release builds; in debug builds,
    /// `data/` at the repository root, which survives `cargo clean` and spares the wiki
    /// repeated downloads during development.
    pub fn default_location() -> std::io::Result<Self> {
        #[cfg(debug_assertions)]
        {
            Ok(Self::new(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("data"),
            ))
        }
        #[cfg(not(debug_assertions))]
        {
            Self::beside_executable()
        }
    }

    /// `data/` next to the executable, as the common base requires.
    pub fn beside_executable() -> std::io::Result<Self> {
        let exe = std::env::current_exe()?;
        let parent = exe.parent().unwrap_or_else(|| Path::new("."));
        Ok(Self::new(parent.join("data")))
    }
}

/// Runs a whole import. `progress` is called before each step; `now` returns Unix seconds.
pub fn run_import(
    paths: &DataPaths,
    origin: Origin,
    now: &dyn Fn() -> i64,
    progress: &mut dyn FnMut(Progress<'_>),
) -> Result<UpdateReport, UpdateError> {
    let total = step_count(origin);
    let mut done = 0;
    let mut step = |message: &str| {
        progress(Progress {
            done,
            total,
            message,
        });
        done += 1;
    };
    std::fs::create_dir_all(&paths.dir).map_err(|source| UpdateError::DataDir {
        path: paths.dir.clone(),
        source,
    })?;
    let cache = ModuleCache::new(&paths.cache);

    if origin == Origin::Wiki {
        let mut fetcher = ModuleFetcher::new()?;
        for (page, file) in MODULES {
            step(&format!("Téléchargement de {page}…"));
            let page_data = fetcher.fetch_raw(page)?;
            cache.store(
                file,
                &CachedModule {
                    text: page_data.text,
                    downloaded_at: now(),
                    last_modified: page_data.last_modified,
                },
            )?;
        }
    }

    step("Lecture des modules…");
    let [drop_tables, missions] = MODULES.map(|(_, file)| cache.load(file));
    let (drop_tables, missions) = (drop_tables?, missions?);
    let revision = revision(&[&drop_tables, &missions]);
    let dataset = WikiSource::new(drop_tables.text, missions.text).load()?;

    step("Écriture dans la base…");
    let mut conn = db::open(&paths.database)?;
    let source = match origin {
        Origin::Wiki => "wiki",
        Origin::Cache => "wiki-cache",
    };
    let summary = db::import_dataset(
        &mut conn,
        &dataset,
        ImportRun {
            source,
            source_revision: Some(&revision),
        },
        now,
    )?;
    Ok(UpdateReport {
        summary,
        warnings: dataset.warnings,
    })
}

/// Identifies the imported module versions: `Last-Modified` when known, else download date.
fn revision(modules: &[&CachedModule]) -> String {
    MODULES
        .iter()
        .zip(modules)
        .map(|((page, _), module)| {
            let when = module
                .last_modified
                .clone()
                .unwrap_or_else(|| format_utc(module.downloaded_at));
            format!("{page} : {when}")
        })
        .collect::<Vec<_>>()
        .join(" ; ")
}

/// Unix seconds to `YYYY-MM-DD HH:MM:SS UTC` (civil-from-days, H. Hinnant), without a date
/// crate for a single format.
pub fn format_utc(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let secs = seconds.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}
