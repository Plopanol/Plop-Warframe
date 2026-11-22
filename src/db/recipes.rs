//! Imported recipes and user data have separate lifetimes across imports.

use std::collections::BTreeMap;

use rusqlite::{Connection, params};

use super::{DbError, NodeFilter, SearchFilter, SpotFilter, search_by_item, spots_for_items};
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

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeSpot {
    pub planet: String,
    pub node: String,
    pub mission_type: String,
    pub min_level: Option<i64>,
    pub max_level: Option<i64>,
    pub chance: f64,
    pub estimated: bool,
    pub enemy: Option<String>,
    pub location: Option<super::Location>,
}

/// Mission rewards and per-kill chances stay separate: they have different denominators.
pub fn recipe_spots(
    conn: &Connection,
    item: &str,
    max_level: Option<u32>,
    mission_type: Option<i64>,
) -> Result<Vec<RecipeSpot>, DbError> {
    use rusqlite::OptionalExtension;
    let item_id: Option<i64> = conn
        .query_row("SELECT id FROM item WHERE name=?1", [item], |r| r.get(0))
        .optional()?;
    let Some(item_id) = item_id else {
        return Ok(Vec::new());
    };
    let mut nodes = conn.prepare("SELECT n.key,n.min_level,n.max_level FROM node n JOIN mission_type mt ON mt.id=n.mission_type_id WHERE n.is_special=0 AND mt.name NOT IN ('Solar Rail Junction','Hub','Relay','Conclave') AND (?1 IS NULL OR n.min_level <= ?1) AND (?2 IS NULL OR n.mission_type_id=?2)")?;
    let allowed: BTreeMap<String, (Option<i64>, Option<i64>)> = nodes
        .query_map(params![max_level, mission_type], |r| {
            Ok((r.get(0)?, (r.get(1)?, r.get(2)?)))
        })?
        .collect::<Result<_, _>>()?;
    let mut spots = Vec::new();
    for row in search_by_item(
        conn,
        &SearchFilter {
            item_ids: vec![item_id],
            mission_type_id: mission_type,
            specials: NodeFilter::Exclude,
            ..Default::default()
        },
    )? {
        if let Some(&(min_level, max_level)) = allowed.get(&row.location.node_key) {
            spots.push(RecipeSpot {
                planet: row.location.planet.clone(),
                node: row.location.node_name.clone(),
                mission_type: row.location.mission_type.clone(),
                min_level,
                max_level,
                chance: row.chance,
                estimated: false,
                enemy: None,
                location: Some(row.location),
            });
        }
    }
    for row in spots_for_items(
        conn,
        &SpotFilter {
            item_ids: vec![item_id],
            mission_type_id: mission_type,
            ..Default::default()
        },
    )? {
        if let Some(&(min_level, max_level)) = allowed.get(&row.node_key) {
            spots.push(RecipeSpot {
                planet: row.planet,
                node: row.node_name,
                mission_type: row.mission_type,
                min_level,
                max_level,
                chance: row.chance,
                estimated: row.estimated,
                enemy: Some(row.best_enemy),
                location: None,
            });
        }
    }
    Ok(spots)
}
