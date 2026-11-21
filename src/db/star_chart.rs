//! Star Chart tab queries: playable nodes of the star chart with their mission type, faction and
//! enemy levels (rule 20 of `docs/04-REGLES-DROPS.md`).

use rusqlite::{Connection, params};

use super::DbError;
use super::search::{MissionTypeRow, NodeFilter};
use super::spot::PlanetRow;

/// Mission types of nodes that are not missions (travel, social spaces, PvP), left out of the
/// star chart list.
pub const EXCLUDED_MISSION_TYPES: &[&str] = &["Solar Rail Junction", "Hub", "Relay", "Conclave"];

/// SQL condition shared by the queries: regular node of a kept mission type (`mt` and `n`).
fn playable() -> String {
    let excluded: Vec<String> = EXCLUDED_MISSION_TYPES
        .iter()
        .map(|name| format!("'{}'", name.replace('\'', "''")))
        .collect();
    format!(
        "n.is_special = 0 AND mt.name NOT IN ({})",
        excluded.join(", ")
    )
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StarChartFilter {
    pub mission_type_id: Option<i64>,
    pub planet_id: Option<i64>,
    /// Faction code of `node_faction` (`grineer`, `corpus`…).
    pub faction: Option<String>,
    /// Bounds of the wanted level range; a node is kept when its range overlaps it.
    pub min_level: Option<i64>,
    pub max_level: Option<i64>,
    pub dark_sectors: NodeFilter,
}

/// Node of the star chart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarChartNode {
    pub id: i64,
    pub name: String,
    pub planet: String,
    /// English wiki name; translation belongs to the UI.
    pub mission_type: String,
    /// Faction text of the wiki (`Grineer`, `Corpus / Grineer`…).
    pub faction: Option<String>,
    pub min_level: Option<i64>,
    pub max_level: Option<i64>,
    pub is_dark_sector: bool,
}

/// Playable nodes matching the filter, by planet then node name.
pub fn star_chart_nodes(
    conn: &Connection,
    filter: &StarChartFilter,
) -> Result<Vec<StarChartNode>, DbError> {
    let playable = playable();
    let sql = format!(
        "SELECT n.id, n.name, p.name, mt.name, n.faction, n.min_level, n.max_level,
                n.is_dark_sector
         FROM node n
         JOIN planet p ON p.id = n.planet_id
         JOIN mission_type mt ON mt.id = n.mission_type_id
         WHERE {playable}
           AND (?1 IS NULL OR n.mission_type_id = ?1)
           AND (?2 IS NULL OR n.planet_id = ?2)
           AND (?3 IS NULL OR EXISTS (
                SELECT 1 FROM node_faction nf WHERE nf.node_id = n.id AND nf.faction = ?3))
           AND (?4 IS NULL OR n.max_level >= ?4)
           AND (?5 IS NULL OR n.min_level <= ?5)
           AND (?6 = 0 OR (?6 = 1 AND n.is_dark_sector = 0) OR (?6 = 2 AND n.is_dark_sector = 1))
         ORDER BY p.name COLLATE NOCASE, n.name COLLATE NOCASE, n.id"
    );
    let mut statement = conn.prepare_cached(&sql)?;
    let rows = statement.query_map(
        params![
            filter.mission_type_id,
            filter.planet_id,
            filter.faction,
            filter.min_level,
            filter.max_level,
            filter.dark_sectors.code(),
        ],
        |row| {
            Ok(StarChartNode {
                id: row.get(0)?,
                name: row.get(1)?,
                planet: row.get(2)?,
                mission_type: row.get(3)?,
                faction: row.get(4)?,
                min_level: row.get(5)?,
                max_level: row.get(6)?,
                is_dark_sector: row.get(7)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Values offered by the Star Chart filters, limited to the playable nodes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StarChartChoices {
    pub mission_types: Vec<MissionTypeRow>,
    pub planets: Vec<PlanetRow>,
    /// Faction codes, sorted.
    pub factions: Vec<String>,
}

pub fn star_chart_choices(conn: &Connection) -> Result<StarChartChoices, DbError> {
    let playable = playable();
    let mut statement = conn.prepare_cached(&format!(
        "SELECT mt.id, mt.name, mt.is_endless, count(n.id)
         FROM node n JOIN mission_type mt ON mt.id = n.mission_type_id
         WHERE {playable}
         GROUP BY mt.id
         ORDER BY mt.name COLLATE NOCASE, mt.name"
    ))?;
    let mission_types = statement
        .query_map([], |row| {
            Ok(MissionTypeRow {
                id: row.get(0)?,
                name: row.get(1)?,
                is_endless: row.get(2)?,
                nodes: row.get(3)?,
                special_nodes: 0,
            })
        })?
        .collect::<Result<_, _>>()?;

    let mut statement = conn.prepare_cached(&format!(
        "SELECT p.id, p.name, count(n.id)
         FROM node n
         JOIN planet p ON p.id = n.planet_id
         JOIN mission_type mt ON mt.id = n.mission_type_id
         WHERE {playable}
         GROUP BY p.id
         ORDER BY p.name COLLATE NOCASE, p.name"
    ))?;
    let planets = statement
        .query_map([], |row| {
            Ok(PlanetRow {
                id: row.get(0)?,
                name: row.get(1)?,
                nodes: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    let mut statement = conn.prepare_cached(&format!(
        "SELECT DISTINCT nf.faction
         FROM node_faction nf
         JOIN node n ON n.id = nf.node_id
         JOIN mission_type mt ON mt.id = n.mission_type_id
         WHERE {playable}
         ORDER BY nf.faction"
    ))?;
    let factions = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;

    Ok(StarChartChoices {
        mission_types,
        planets,
        factions,
    })
}
