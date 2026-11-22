//! Import use case shared by the CLI and the UI: download (or read the cache), parse, store.
//! Blocking: run it on a worker thread.

use crate::source::{BLUEPRINTS_CACHE, BLUEPRINTS_MODULE, BlueprintSource};
use std::path::{Path, PathBuf};

use crate::db::{self, DbError, ImportRun, ImportSummary};
use crate::source::{
    CacheError, CachedModule, DE_DROP_TABLES, DE_DROP_TABLES_URL, DeRelicSource, DropSource,
    ENEMY_FACTIONS, FetchError, ModuleCache, ModuleFetcher, SourceError, WFCD_CATEGORIES,
    WFCD_FR_FILE, WFCD_RELICS, WFCD_RELICS_CATEGORY, WfcdItemsSource, WfcdSource, WikiSource,
    wfcd_category_file, wfcd_item_files,
};

/// Wiki page and cache file of each required module, in download order.
pub const MODULES: [(&str, &str); 2] = [
    ("Module:DropTables/data", "droptables_data.lua"),
    ("Module:Missions/data", "missions_data.lua"),
];

/// Wiki page and cache file of each enemy location module. They are optional in the cache, so
/// a cache made before them still imports, with enemies left without a spot.
pub fn enemy_modules() -> Vec<(String, String)> {
    ENEMY_FACTIONS
        .iter()
        .map(|faction| {
            (
                format!("Module:Enemies/data/{faction}"),
                format!("enemies_{faction}.lua"),
            )
        })
        .collect()
}

/// Cache file of the DE drop tables page, read for the relic contents (ADR-002). Optional in
/// the cache, so a cache made before it still imports, without relics.
pub const DE_CACHE_FILE: &str = "droptables_de.html";

/// Former cache file of the WFCD relic list, still read for the vaulted status (ADR-003) when
/// `wfcd/Relics.json` is missing, so a cache made before ADR-004 keeps its status.
pub const WFCD_CACHE_FILE: &str = "wfcd_relics.json";

/// Cache subdirectory of the `@wfcd/items` files (ADR-004). Every file is optional.
pub const WFCD_CACHE_DIR: &str = "wfcd";

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
        Origin::Wiki => MODULES.len() + ENEMY_FACTIONS.len() + 4 + WFCD_CATEGORIES.len() + 1,
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
    let wfcd_cache = ModuleCache::new(paths.cache.join(WFCD_CACHE_DIR));
    let mut cache_warnings = Vec::new();

    let enemy_modules = enemy_modules();
    if origin == Origin::Wiki {
        let mut fetcher = ModuleFetcher::new()?;
        let pages = MODULES.iter().map(|(page, file)| (*page, *file)).chain(
            enemy_modules
                .iter()
                .map(|(page, file)| (page.as_str(), file.as_str())),
        );
        for (page, file) in pages {
            step(&format!("Téléchargement de {page}…"));
            let page_data = fetcher.fetch_raw(page)?;
            cache.store(
                file,
                &CachedModule {
                    text: page_data.text,
                    downloaded_at: now(),
                    last_modified: page_data.last_modified,
                    version: page_data.version,
                },
            )?;
        }
        step("Téléchargement des recettes…");
        match fetcher.fetch_raw(BLUEPRINTS_MODULE) {
            Ok(page) => cache.store(
                BLUEPRINTS_CACHE,
                &CachedModule {
                    text: page.text,
                    downloaded_at: now(),
                    last_modified: page.last_modified,
                    version: page.version,
                },
            )?,
            Err(error) => cache_warnings.push(format!(
                "{error} ; recettes précédentes du cache conservées si présentes"
            )),
        }
        step(&format!("Téléchargement des {DE_DROP_TABLES}…"));
        let page_data = fetcher.fetch_url(DE_DROP_TABLES, DE_DROP_TABLES_URL)?;
        cache.store(
            DE_CACHE_FILE,
            &CachedModule {
                text: page_data.text,
                downloaded_at: now(),
                last_modified: page_data.last_modified,
                version: page_data.version,
            },
        )?;
        for (file, url) in wfcd_item_files() {
            let label = format!("@wfcd/items {file}");
            step(&format!("Téléchargement de {label}…"));
            // A WFCD file is optional: the previous copy of the cache stays in use.
            match fetcher.fetch_url(&label, &url) {
                Ok(page_data) => wfcd_cache.store(
                    &file,
                    &CachedModule {
                        text: page_data.text,
                        downloaded_at: now(),
                        last_modified: page_data.last_modified,
                        version: page_data.version,
                    },
                )?,
                Err(error) => cache_warnings.push(format!(
                    "{error} ; copie précédente du cache conservée si elle existe"
                )),
            }
        }
    }

    step("Lecture des modules…");
    let [drop_tables, missions] = MODULES.map(|(_, file)| cache.load(file));
    let (drop_tables, missions) = (drop_tables?, missions?);
    let mut revision = revision(&[&drop_tables, &missions]);
    let mut enemy_texts = Vec::new();
    for (page, file) in enemy_modules {
        match cache.load(&file) {
            Ok(module) => enemy_texts.push((page, module.text)),
            Err(CacheError::Missing(_)) => cache_warnings.push(format!(
                "Module « {page} » absent du cache : relancer une mise à jour depuis le wiki"
            )),
            Err(error) => return Err(error.into()),
        }
    }
    let mut dataset = WikiSource::new(drop_tables.text, missions.text)
        .with_enemy_modules(enemy_texts)
        .load()?;
    match cache.load(BLUEPRINTS_CACHE) {
        Ok(module) => {
            let recipes = BlueprintSource(&module.text).load()?;
            dataset.recipes = recipes.recipes;
            dataset.warnings.extend(recipes.warnings);
            revision.push_str(&format!(" ; {BLUEPRINTS_MODULE} : {}", when(&module)));
        }
        Err(CacheError::Missing(_)) => cache_warnings.push(
            "Recettes absentes du cache : mettez à jour depuis le wiki pour charger le catalogue."
                .to_owned(),
        ),
        Err(error) => return Err(error.into()),
    }
    match cache.load(DE_CACHE_FILE) {
        Ok(page) => {
            let relics = DeRelicSource::new(page.text.as_str()).load()?;
            dataset.relic_rewards = relics.relic_rewards;
            dataset.warnings.extend(relics.warnings);
            revision.push_str(&format!(" ; {DE_DROP_TABLES} : {}", when(&page)));
        }
        Err(CacheError::Missing(_)) => cache_warnings.push(format!(
            "{DE_DROP_TABLES} absentes du cache, aucune relique : relancer une mise à jour"
        )),
        Err(error) => return Err(error.into()),
    }
    let wfcd = load_wfcd(&wfcd_cache, &mut cache_warnings)?;
    let relics_text = wfcd
        .files
        .iter()
        .find(|(category, _)| category == WFCD_RELICS_CATEGORY)
        .map(|(_, text)| text.as_str());
    let legacy_relics = match relics_text {
        Some(_) => None,
        None => match cache.load(WFCD_CACHE_FILE) {
            Ok(list) => Some(list),
            Err(CacheError::Missing(_)) => None,
            Err(error) => return Err(error.into()),
        },
    };
    match relics_text.or(legacy_relics.as_ref().map(|list| list.text.as_str())) {
        Some(text) => {
            let status = WfcdSource::new(text).load()?;
            dataset.relic_status = status.relic_status;
            dataset.warnings.extend(status.warnings);
        }
        None => cache_warnings.push(format!(
            "{WFCD_RELICS} absente du cache, statut vaulted inconnu : relancer une mise à jour"
        )),
    }
    match (&wfcd.revision, &legacy_relics) {
        (Some(wfcd_revision), _) => revision.push_str(&format!(" ; {wfcd_revision}")),
        (None, Some(list)) => revision.push_str(&format!(" ; {WFCD_RELICS} : {}", when(list))),
        (None, None) => {}
    }
    let items = WfcdItemsSource::new(wfcd.files, wfcd.fr).load()?;
    dataset.wfcd_items = items.wfcd_items;
    dataset.warnings.extend(items.warnings);
    dataset.warnings.splice(0..0, cache_warnings);

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

/// `@wfcd/items` files found in the cache.
struct WfcdFiles {
    /// (category, JSON text), in the order of `WFCD_CATEGORIES`.
    files: Vec<(String, String)>,
    fr: Option<String>,
    /// `WFCD <version> : <date>` of `Relics.json`, else of the first file read.
    revision: Option<String>,
}

/// Reads the cached `@wfcd/items` files; a missing one gives a warning, not an error.
fn load_wfcd(cache: &ModuleCache, warnings: &mut Vec<String>) -> Result<WfcdFiles, CacheError> {
    let mut wfcd = WfcdFiles {
        files: Vec::new(),
        fr: None,
        revision: None,
    };
    let mut first_read = None;
    let mut missing = Vec::new();
    let files = WFCD_CATEGORIES
        .iter()
        .map(|category| (Some(*category), wfcd_category_file(category)))
        .chain(std::iter::once((None, WFCD_FR_FILE.to_owned())));
    for (category, file) in files {
        let module = match cache.load(&file) {
            Ok(module) => module,
            Err(CacheError::Missing(_)) => {
                missing.push(file);
                continue;
            }
            Err(error) => return Err(error),
        };
        let module_revision = wfcd_revision(&module);
        if category == Some(WFCD_RELICS_CATEGORY) {
            wfcd.revision = Some(module_revision);
        } else if first_read.is_none() {
            first_read = Some(module_revision);
        }
        match category {
            Some(category) => wfcd.files.push((category.to_owned(), module.text)),
            None => wfcd.fr = Some(module.text),
        }
    }
    if wfcd.revision.is_none() {
        wfcd.revision = first_read;
    }
    // One line for the whole set: a cache made before ADR-004 would otherwise log 27 lines.
    if !missing.is_empty() {
        warnings.push(format!(
            "Fichiers WFCD absents du cache ({}) : relancer une mise à jour",
            missing.join(", ")
        ));
    }
    Ok(wfcd)
}

fn wfcd_revision(module: &CachedModule) -> String {
    match &module.version {
        Some(version) => format!("WFCD {version} : {}", when(module)),
        None => format!("WFCD : {}", when(module)),
    }
}

/// Identifies the imported module versions: `Last-Modified` when known, else download date.
fn revision(modules: &[&CachedModule]) -> String {
    MODULES
        .iter()
        .zip(modules)
        .map(|((page, _), module)| format!("{page} : {}", when(module)))
        .collect::<Vec<_>>()
        .join(" ; ")
}

/// `Last-Modified` of a cached page when known, else its download date.
fn when(module: &CachedModule) -> String {
    module
        .last_modified
        .clone()
        .unwrap_or_else(|| format_utc(module.downloaded_at))
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
