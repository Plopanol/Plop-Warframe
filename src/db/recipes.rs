//! Imported recipes and user data have separate lifetimes across imports.

use std::collections::BTreeMap;

use rusqlite::{Connection, params};

use super::DbError;
use crate::recipes::Recipe;

pub fn recipes(conn: &Connection) -> Result<BTreeMap<String, Recipe>, DbError> {
    let mut query = conn.prepare("SELECT json FROM recipe ORDER BY result")?;
    let rows = query.query_map([], |row| row.get::<_, String>(0))?;
    let mut result = BTreeMap::new();
    for row in rows {
        let recipe: Recipe = serde_json::from_str(&row?)?;
        result.insert(recipe.result.clone(), recipe);
    }
    // Local edits intentionally win over the imported catalogue.
    let mut query = conn.prepare("SELECT json FROM local_recipe")?;
    for row in query.query_map([], |row| row.get::<_, String>(0))? {
        let recipe: Recipe = serde_json::from_str(&row?)?;
        result.insert(recipe.result.clone(), recipe);
    }
    Ok(result)
}

pub fn save_local_recipe(conn: &Connection, recipe: &Recipe) -> Result<(), DbError> {
    recipe.validate()?;
    conn.execute("INSERT INTO local_recipe(result,json) VALUES (?1,?2) ON CONFLICT(result) DO UPDATE SET json=excluded.json", params![recipe.result, serde_json::to_string(recipe)?])?;
    Ok(())
}

pub fn profile_level(conn: &Connection) -> Result<Option<u32>, DbError> {
    Ok(conn.query_row(
        "SELECT max_enemy_level FROM character_profile WHERE id=1",
        [],
        |r| r.get(0),
    )?)
}

pub fn save_profile_level(conn: &Connection, level: Option<u32>) -> Result<(), DbError> {
    if level == Some(0) {
        return Err(crate::recipes::RecipeError::Invalid.into());
    }
    conn.execute(
        "UPDATE character_profile SET max_enemy_level=?1 WHERE id=1",
        [level],
    )?;
    Ok(())
}
