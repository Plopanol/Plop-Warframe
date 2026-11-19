//! Disk cache of downloaded modules, so an import can be replayed without network.
//! Layout: `docs/05-DATA-SCHEMAS.md`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("cache absent : « {0} » (lancez d'abord une mise à jour depuis le wiki)")]
    Missing(PathBuf),
    #[error("cache « {path} » : {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedModule {
    pub text: String,
    /// Unix seconds.
    pub downloaded_at: i64,
    pub last_modified: Option<String>,
}

pub struct ModuleCache {
    dir: PathBuf,
}

impl ModuleCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Writes the text and its metadata. The text goes to a temporary file renamed at the
    /// end, so an interrupted download never leaves a truncated module behind.
    pub fn store(&self, file_name: &str, module: &CachedModule) -> Result<(), CacheError> {
        let io_error = |path: &Path| {
            let path = path.to_owned();
            move |source| CacheError::Io { path, source }
        };
        fs::create_dir_all(&self.dir).map_err(io_error(&self.dir))?;

        let path = self.dir.join(file_name);
        let partial = self.dir.join(format!("{file_name}.part"));
        fs::write(&partial, &module.text).map_err(io_error(&partial))?;
        fs::rename(&partial, &path).map_err(io_error(&path))?;

        let meta_path = meta_path(&path);
        let mut meta = format!("downloaded_at={}\n", module.downloaded_at);
        if let Some(last_modified) = &module.last_modified {
            meta.push_str(&format!("last_modified={last_modified}\n"));
        }
        fs::write(&meta_path, meta).map_err(io_error(&meta_path))
    }

    pub fn load(&self, file_name: &str) -> Result<CachedModule, CacheError> {
        let path = self.dir.join(file_name);
        if !path.is_file() {
            return Err(CacheError::Missing(path));
        }
        let text = fs::read_to_string(&path).map_err(|source| CacheError::Io {
            path: path.clone(),
            source,
        })?;
        // Metadata is informative only: a missing or damaged file must not block an import.
        let meta = fs::read_to_string(meta_path(&path)).unwrap_or_default();
        let mut downloaded_at = 0;
        let mut last_modified = None;
        for line in meta.lines() {
            if let Some(value) = line.strip_prefix("downloaded_at=") {
                downloaded_at = value.trim().parse().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("last_modified=") {
                last_modified = Some(value.trim().to_owned());
            }
        }
        Ok(CachedModule {
            text,
            downloaded_at,
            last_modified,
        })
    }
}

fn meta_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".meta");
    PathBuf::from(name)
}
