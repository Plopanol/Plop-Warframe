//! Restricted Lua interpreter for wiki data modules.

use mlua::{Lua, LuaOptions, LuaSerdeExt, StdLib, Value};
use serde::de::DeserializeOwned;

/// The modules are plain data plus a little post-processing; 512 MiB is far above their needs
/// and only stops a runaway module.
const MEMORY_LIMIT: usize = 512 * 1024 * 1024;

/// Globals of the base library that reach the file system or load arbitrary code.
const REMOVED_GLOBALS: [&str; 4] = ["dofile", "loadfile", "load", "require"];

/// Lua failure, kept as text: `mlua::Error` is neither `Send` nor `Sync`, and errors must
/// travel from the import thread to the UI.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct LuaError(String);

impl From<mlua::Error> for LuaError {
    fn from(error: mlua::Error) -> Self {
        Self(error.to_string())
    }
}

/// Lua 5.4 without `io`, `os`, `package` nor `debug`, because the code comes from an editable wiki.
pub struct SandboxedLua {
    lua: Lua,
}

impl SandboxedLua {
    pub fn new() -> Result<Self, LuaError> {
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
            LuaOptions::default(),
        )?;
        lua.set_memory_limit(MEMORY_LIMIT)?;
        let globals = lua.globals();
        for name in REMOVED_GLOBALS {
            globals.set(name, Value::Nil)?;
        }
        Ok(Self { lua })
    }

    /// Runs a module chunk and returns the value of its final `return`.
    pub fn run_module(&self, name: &str, code: &str) -> Result<Value, LuaError> {
        Ok(self.lua.load(code).set_name(name).eval::<Value>()?)
    }

    /// Reads `value[key]`, or `nil` when `value` is not a table. Taking single fields avoids
    /// walking the modules' reverse indexes, which hold references back into the data.
    pub fn field(&self, value: &Value, key: &str) -> Result<Value, LuaError> {
        match value.as_table() {
            Some(table) => Ok(table.get::<Value>(key)?),
            None => Ok(Value::Nil),
        }
    }

    /// Converts a Lua value; functions inside tables are skipped rather than rejected,
    /// because the modules mix helper functions with data.
    pub fn convert<T: DeserializeOwned>(&self, value: Value) -> Result<T, LuaError> {
        let options = mlua::serde::DeserializeOptions::new().deny_unsupported_types(false);
        Ok(self.lua.from_value_with(value, options)?)
    }

    /// String-keyed entries of a table, in no particular order; empty for any other value.
    pub fn entries(&self, value: &Value) -> Result<Vec<(String, Value)>, LuaError> {
        let Some(table) = value.as_table() else {
            return Ok(Vec::new());
        };
        let mut entries = Vec::new();
        for pair in table.pairs::<Value, Value>() {
            let (key, value) = pair?;
            if let Value::String(key) = key {
                entries.push((key.to_str()?.to_owned(), value));
            }
        }
        Ok(entries)
    }

    /// Text of a string value, `None` for anything else.
    pub fn text(&self, value: &Value) -> Option<String> {
        match value {
            Value::String(text) => text.to_str().ok().map(|text| text.to_owned()),
            _ => None,
        }
    }

    /// Strings of a list (its sequence part) or of a single string; other values are skipped,
    /// because an empty Lua table cannot be told apart from an empty map.
    pub fn texts(&self, value: &Value) -> Result<Vec<String>, LuaError> {
        match value {
            Value::Table(table) => {
                let mut texts = Vec::new();
                for item in table.sequence_values::<Value>() {
                    if let Some(text) = self.text(&item?) {
                        texts.push(text);
                    }
                }
                Ok(texts)
            }
            other => Ok(self.text(other).into_iter().collect()),
        }
    }

    /// Evaluates an expression; used by tests to inspect the sandbox.
    pub fn eval<T: mlua::FromLua>(&self, expression: &str) -> Result<T, LuaError> {
        Ok(self.lua.load(expression).eval::<T>()?)
    }
}
