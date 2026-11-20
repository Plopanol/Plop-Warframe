//! Queries of the Items tab on the `@wfcd/items` items (`wfcd_item` table, ADR-004).

use rusqlite::{Connection, OptionalExtension, params};

use super::{DbError, fold_text};

/// Item found by [`search_wfcd_items`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfcdItemHit {
    pub id: i64,
    pub name: String,
    /// `None` when the translation lacks it or gives an empty name (some skins).
    pub name_fr: Option<String>,
    pub category: String,
}

/// Distinct categories present in the base, unordered: the display order belongs to the view.
pub fn wfcd_categories(conn: &Connection) -> Result<Vec<String>, DbError> {
    let mut statement = conn.prepare_cached("SELECT DISTINCT category FROM wfcd_item")?;
    let rows = statement.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Items of `category` (every category when `None`) whose English or French name contains
/// `query` without case or accents; at most `limit` of them, with the count of every match.
/// Names starting with the query come first, then by shown name (French when known).
pub fn search_wfcd_items(
    conn: &Connection,
    category: Option<&str>,
    query: &str,
    limit: u32,
) -> Result<(Vec<WfcdItemHit>, usize), DbError> {
    let needle = fold_text(query.trim());
    // `instr` rather than LIKE: no wildcard to escape in the user's text. `count(*) OVER ()`
    // is computed before LIMIT, so it gives the number of every match in the same query.
    let mut statement = conn.prepare_cached(
        "SELECT id, name, nullif(name_fr, ''), category, count(*) OVER ()
         FROM wfcd_item
         WHERE (?2 IS NULL OR category = ?2)
           AND (instr(fold(name), ?1) > 0 OR instr(fold(name_fr), ?1) > 0)
         ORDER BY instr(fold(ifnull(nullif(name_fr, ''), name)), ?1) <> 1,
                  ifnull(nullif(name_fr, ''), name) COLLATE NOCASE, name, id
         LIMIT ?3",
    )?;
    let mut total = 0_i64;
    let rows = statement.query_map(params![needle, category, i64::from(limit)], |row| {
        Ok((
            WfcdItemHit {
                id: row.get(0)?,
                name: row.get(1)?,
                name_fr: row.get(2)?,
                category: row.get(3)?,
            },
            row.get::<_, i64>(4)?,
        ))
    })?;
    let mut hits = Vec::new();
    for row in rows {
        let (hit, count) = row?;
        total = count;
        hits.push(hit);
    }
    Ok((hits, usize::try_from(total).unwrap_or(0)))
}

/// Raw JSON of an item and its French translation, `None` when the id is unknown.
pub fn wfcd_item_detail(
    conn: &Connection,
    id: i64,
) -> Result<Option<(String, Option<String>)>, DbError> {
    Ok(conn
        .prepare_cached("SELECT json, json_fr FROM wfcd_item WHERE id = ?1")?
        .query_row(params![id], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()?)
}
