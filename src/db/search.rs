//! Search queries for the Search tab: item autocompletion, mission types, and drop locations
//! "by item" or "combined". Filtering and aggregation run in SQL; Rust only binds parameters
//! and reads rows. Rules: `docs/04-REGLES-DROPS.md` (rules 5 and 10).

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use rusqlite::{Connection, Row, params};

use super::DbError;
use crate::model::{Rarity, Rotation, RotationKind, TableRole};

/// Lowercases `text` and strips the accents of Latin letters, so that a search for "hollars"
/// finds "Höllars Cache". Also registered as the SQL function `fold`.
pub fn fold_text(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    for c in text.chars().flat_map(char::to_lowercase) {
        match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => folded.push('a'),
            'æ' => folded.push_str("ae"),
            'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => folded.push('c'),
            'ď' | 'đ' => folded.push('d'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => folded.push('e'),
            'ĝ' | 'ğ' | 'ġ' | 'ģ' => folded.push('g'),
            'ĥ' | 'ħ' => folded.push('h'),
            'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => folded.push('i'),
            'ĵ' => folded.push('j'),
            'ķ' => folded.push('k'),
            'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => folded.push('l'),
            'ñ' | 'ń' | 'ņ' | 'ň' => folded.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => folded.push('o'),
            'œ' => folded.push_str("oe"),
            'ŕ' | 'ŗ' | 'ř' => folded.push('r'),
            'ś' | 'ŝ' | 'ş' | 'š' => folded.push('s'),
            'ß' => folded.push_str("ss"),
            'ţ' | 'ť' | 'ŧ' => folded.push('t'),
            'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => folded.push('u'),
            'ŵ' => folded.push('w'),
            'ý' | 'ÿ' | 'ŷ' => folded.push('y'),
            'ź' | 'ż' | 'ž' => folded.push('z'),
            other => folded.push(other),
        }
    }
    folded
}

/// Item proposed by the autocompletion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemHit {
    pub id: i64,
    pub name: String,
    pub category: String,
}

/// Items whose name contains `text`, ignoring case and accents, restricted to `category` when
/// given; prefix matches first, then by name. An empty text yields every item of `category`,
/// or nothing without a category.
pub fn search_items(
    conn: &Connection,
    text: &str,
    category: Option<&str>,
    limit: u32,
) -> Result<Vec<ItemHit>, DbError> {
    let needle = fold_text(text.trim());
    if needle.is_empty() && category.is_none() {
        return Ok(Vec::new());
    }
    // `instr` rather than LIKE: no wildcard to escape in the user's text. An empty needle
    // matches at position 1 for every name.
    let mut statement = conn.prepare_cached(
        "SELECT id, name, category FROM item
         WHERE instr(fold(name), ?1) > 0 AND (?3 IS NULL OR category = ?3)
         ORDER BY instr(fold(name), ?1) <> 1, name COLLATE NOCASE, name
         LIMIT ?2",
    )?;
    let rows = statement.query_map(params![needle, i64::from(limit), category], |row| {
        Ok(ItemHit {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Distinct item categories, the most populated first, for the item type filter.
pub fn item_categories(conn: &Connection) -> Result<Vec<String>, DbError> {
    let mut statement = conn.prepare_cached(
        "SELECT category FROM item GROUP BY category ORDER BY count(*) DESC, category",
    )?;
    let rows = statement.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Mission type with its node counts, for the mission type filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionTypeRow {
    pub id: i64,
    /// English wiki name; translation belongs to the UI.
    pub name: String,
    pub is_endless: bool,
    pub nodes: i64,
    pub special_nodes: i64,
}

pub fn mission_types(conn: &Connection) -> Result<Vec<MissionTypeRow>, DbError> {
    let mut statement = conn.prepare_cached(
        "SELECT mt.id, mt.name, mt.is_endless, count(n.id), ifnull(sum(n.is_special), 0)
         FROM mission_type mt LEFT JOIN node n ON n.mission_type_id = mt.id
         GROUP BY mt.id
         ORDER BY mt.name COLLATE NOCASE, mt.name",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(MissionTypeRow {
            id: row.get(0)?,
            name: row.get(1)?,
            is_endless: row.get(2)?,
            nodes: row.get(3)?,
            special_nodes: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Rotations kept; only applies to tables whose pools are rotations (rule 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotationFilter {
    pub a: bool,
    pub b: bool,
    pub c: bool,
}

impl RotationFilter {
    pub const ALL: Self = Self {
        a: true,
        b: true,
        c: true,
    };
}

impl Default for RotationFilter {
    fn default() -> Self {
        Self::ALL
    }
}

/// Treatment of special missions (bounties, Sortie, Arbitrations…).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SpecialFilter {
    #[default]
    Include,
    Exclude,
    Only,
}

impl SpecialFilter {
    fn code(self) -> i64 {
        match self {
            SpecialFilter::Include => 0,
            SpecialFilter::Exclude => 1,
            SpecialFilter::Only => 2,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchFilter {
    /// `None` searches every mission type.
    pub mission_type_id: Option<i64>,
    pub rotations: RotationFilter,
    pub specials: SpecialFilter,
    /// Chosen items; none gives no result.
    pub item_ids: Vec<i64>,
}

/// Where a result was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub node_key: String,
    pub node_name: String,
    pub planet: String,
    pub mission_type: String,
    pub is_special: bool,
    pub table_alias: String,
    pub table_name: String,
    pub rotation_kind: RotationKind,
    pub role: TableRole,
    pub rotation: Rotation,
}

/// "By item" mode: one row per drop entry of a chosen item.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemDropRow {
    pub location: Location,
    pub item_id: i64,
    pub item_name: String,
    /// Fraction between 0 and 1.
    pub chance: f64,
    pub rarity: Rarity,
    pub quantity: u32,
    /// Chance per AABC cycle of this item in this table, for a rotation table of an endless
    /// mission type; the same on the A, B and C rows.
    pub aabc: Option<f64>,
}

/// "Combined" mode: one row per node, table and rotation, summing the chosen items.
#[derive(Debug, Clone, PartialEq)]
pub struct CombinedRow {
    pub location: Location,
    /// Sum of the chances of the chosen items found in this pool (one draw yields one item).
    pub chance: f64,
    /// Number of distinct chosen items found in this pool.
    pub items: u32,
    /// Combined chance per AABC cycle, as for [`ItemDropRow::aabc`].
    pub aabc: Option<f64>,
}

/// Chosen drops of the filter: `?1` is the JSON array of item ids.
const CHOSEN_CTE: &str = "chosen AS (
        SELECT drop_table_id, rotation, item_id, chance, rarity, quantity FROM \"drop\"
        WHERE item_id IN (SELECT value FROM json_each(?1))
    )";

/// Joins `r` (columns `drop_table_id`, `rotation`) and `a` (`aabc`) to their nodes and applies
/// the filters `?2` mission type, `?3`..`?5` rotations A, B, C, `?6` special missions.
const LOCATION_JOINS: &str = "
    JOIN drop_table dt ON dt.id = r.drop_table_id
    JOIN node_drop_table ndt ON ndt.drop_table_id = dt.id
    JOIN node n ON n.id = ndt.node_id
    JOIN planet p ON p.id = n.planet_id
    JOIN mission_type mt ON mt.id = n.mission_type_id
    WHERE (?2 IS NULL OR n.mission_type_id = ?2)
      AND (dt.rotation_kind <> 'rotation'
           OR (r.rotation = 'A' AND ?3) OR (r.rotation = 'B' AND ?4) OR (r.rotation = 'C' AND ?5))
      AND (?6 = 0 OR (?6 = 1 AND n.is_special = 0) OR (?6 = 2 AND n.is_special = 1))";

/// Location columns, read by [`location`] from index 0.
const LOCATION_COLUMNS: &str = "n.key, n.name, p.name, mt.name, n.is_special, dt.alias, dt.name,
    dt.rotation_kind, ndt.role, r.rotation,
    CASE WHEN dt.rotation_kind = 'rotation' AND mt.is_endless = 1 THEN a.aabc END";

/// `(2·A + B + C) / 4` over a `chance` column grouped by rotation.
const AABC_EXPR: &str = "(2 * sum(CASE WHEN rotation = 'A' THEN chance ELSE 0 END)
        + sum(CASE WHEN rotation = 'B' THEN chance ELSE 0 END)
        + sum(CASE WHEN rotation = 'C' THEN chance ELSE 0 END)) / 4.0";

const LOCATION_COLUMN_COUNT: usize = 11;

pub fn search_by_item(
    conn: &Connection,
    filter: &SearchFilter,
) -> Result<Vec<ItemDropRow>, DbError> {
    if filter.item_ids.is_empty() {
        return Ok(Vec::new());
    }
    // The AABC chance is computed before the rotation filter, over every rotation of the table.
    let sql = format!(
        "WITH {CHOSEN_CTE},
         aabc AS (SELECT drop_table_id, item_id, {AABC_EXPR} AS aabc
                  FROM chosen GROUP BY drop_table_id, item_id)
         SELECT {LOCATION_COLUMNS}, i.id, i.name, r.chance, r.rarity, r.quantity
         FROM chosen r
         JOIN aabc a ON a.drop_table_id = r.drop_table_id AND a.item_id = r.item_id
         JOIN item i ON i.id = r.item_id
         {LOCATION_JOINS}
         ORDER BY r.chance DESC, n.name, n.key, dt.alias, ndt.role, r.rotation, i.name,
                  r.quantity"
    );
    let mut statement = conn.prepare_cached(&sql)?;
    let rows = statement.query_map(
        params![
            item_ids_json(filter),
            filter.mission_type_id,
            filter.rotations.a,
            filter.rotations.b,
            filter.rotations.c,
            filter.specials.code()
        ],
        |row| {
            let base = LOCATION_COLUMN_COUNT;
            Ok(ItemDropRow {
                location: location(row)?,
                aabc: row.get(base - 1)?,
                item_id: row.get(base)?,
                item_name: row.get(base + 1)?,
                chance: row.get(base + 2)?,
                rarity: row.get::<_, Code<Rarity>>(base + 3)?.0,
                quantity: row.get(base + 4)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn search_combined(
    conn: &Connection,
    filter: &SearchFilter,
) -> Result<Vec<CombinedRow>, DbError> {
    if filter.item_ids.is_empty() {
        return Ok(Vec::new());
    }
    // Chances depend on the table only, so they are summed per table before the node join.
    let sql = format!(
        "WITH {CHOSEN_CTE},
         pooled AS (SELECT drop_table_id, rotation, sum(chance) AS chance,
                           count(DISTINCT item_id) AS items
                    FROM chosen GROUP BY drop_table_id, rotation),
         aabc AS (SELECT drop_table_id, {AABC_EXPR} AS aabc FROM pooled GROUP BY drop_table_id)
         SELECT {LOCATION_COLUMNS}, r.chance, r.items
         FROM pooled r
         JOIN aabc a ON a.drop_table_id = r.drop_table_id
         {LOCATION_JOINS}
         ORDER BY r.chance DESC, n.name, n.key, dt.alias, ndt.role, r.rotation"
    );
    let mut statement = conn.prepare_cached(&sql)?;
    let rows = statement.query_map(
        params![
            item_ids_json(filter),
            filter.mission_type_id,
            filter.rotations.a,
            filter.rotations.b,
            filter.rotations.c,
            filter.specials.code()
        ],
        |row| {
            let base = LOCATION_COLUMN_COUNT;
            Ok(CombinedRow {
                location: location(row)?,
                aabc: row.get(base - 1)?,
                chance: row.get(base)?,
                items: row.get(base + 1)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// JSON array of the chosen item ids, bound as `?1`. Ids are integers: no escaping needed.
fn item_ids_json(filter: &SearchFilter) -> String {
    let ids: Vec<String> = filter.item_ids.iter().map(i64::to_string).collect();
    format!("[{}]", ids.join(","))
}

fn location(row: &Row<'_>) -> rusqlite::Result<Location> {
    Ok(Location {
        node_key: row.get(0)?,
        node_name: row.get(1)?,
        planet: row.get(2)?,
        mission_type: row.get(3)?,
        is_special: row.get(4)?,
        table_alias: row.get(5)?,
        table_name: row.get(6)?,
        rotation_kind: row.get::<_, Code<RotationKind>>(7)?.0,
        role: row.get::<_, Code<TableRole>>(8)?.0,
        rotation: row.get::<_, Code<Rotation>>(9)?.0,
    })
}

/// Reads a database code into its domain enum.
struct Code<T>(T);

#[derive(Debug, thiserror::Error)]
#[error("code inconnu en base : {0}")]
struct UnknownCode(String);

macro_rules! code_from_sql {
    ($($kind:ty),*) => {$(
        impl FromSql for Code<$kind> {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                let text = value.as_str()?;
                <$kind>::from_code(text)
                    .map(Code)
                    .ok_or_else(|| FromSqlError::Other(Box::new(UnknownCode(text.to_owned()))))
            }
        }
    )*};
}

code_from_sql!(Rarity, Rotation, RotationKind, TableRole);
