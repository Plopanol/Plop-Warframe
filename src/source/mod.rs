//! Data sources. Every source goes through [`DropSource`] so the wiki can be replaced later.

mod lua;
mod wiki;

pub use lua::{LuaError, SandboxedLua};
pub use wiki::WikiSource;

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
