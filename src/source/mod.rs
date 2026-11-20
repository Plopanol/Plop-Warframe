//! Data sources: the wiki for missions and enemies, the DE drop tables for relic contents,
//! `@wfcd/items` for every item and the vaulted status of relics.
//! Every source goes through [`DropSource`] so a source can be replaced later.

mod cache;
mod de;
mod http;
mod lua;
mod wfcd;
mod wiki;

pub use cache::{CacheError, CachedModule, ModuleCache};
pub use de::{DE_DROP_TABLES, DE_DROP_TABLES_URL, DeRelicSource, parse_relic_rewards};
pub use http::{FetchError, FetchedPage, ModuleFetcher, WIKI_BASE_URL};
pub use lua::{LuaError, SandboxedLua};
pub use wfcd::{
    WFCD_CATEGORIES, WFCD_FR_FILE, WFCD_FR_PATH, WFCD_ITEMS, WFCD_ITEMS_BASE_URL, WFCD_RELICS,
    WFCD_RELICS_CATEGORY, WfcdItemsSource, WfcdSource, parse_relic_vaulted, wfcd_category_file,
    wfcd_item_files,
};
pub use wiki::{ENEMY_FACTIONS, WikiSource};

use crate::model::DropDataset;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("exécution du module « {module} » impossible : {source}")]
    Lua {
        module: &'static str,
        #[source]
        source: LuaError,
    },
    #[error("structure inattendue dans « {module} » : {message}")]
    Structure {
        module: &'static str,
        message: String,
    },
}

pub trait DropSource {
    /// Produces the whole dataset; anomalies that do not prevent an import go to `warnings`.
    fn load(&self) -> Result<DropDataset, SourceError>;
}
