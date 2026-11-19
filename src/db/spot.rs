//! Queries of the Spot Search tab: enemies dropping the chosen items, and the nodes where they
//! appear. Rules: `docs/04-REGLES-DROPS.md` (rules 12 to 14).

use rusqlite::{Connection, params};

use super::DbError;

/// Planet holding regular nodes, for the planet filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanetRow {
    pub id: i64,
    pub name: String,
    pub nodes: i64,
}

/// Planets with at least one regular node, by name.
pub fn planets(conn: &Connection) -> Result<Vec<PlanetRow>, DbError> {
    let mut statement = conn.prepare_cached(
        "SELECT p.id, p.name, count(n.id)
         FROM planet p JOIN node n ON n.planet_id = p.id AND n.is_special = 0
         GROUP BY p.id
         ORDER BY p.name COLLATE NOCASE, p.name",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(PlanetRow {
            id: row.get(0)?,
            name: row.get(1)?,
            nodes: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpotFilter {
    /// Chosen items; none gives no result.
    pub item_ids: Vec<i64>,
    pub mission_type_id: Option<i64>,
    pub planet_id: Option<i64>,
    /// Restricts the spots to one enemy (the one picked in the enemy table).
    pub enemy_id: Option<i64>,
}

/// One enemy and one chosen item.
#[derive(Debug, Clone, PartialEq)]
pub struct EnemyDropRow {
    pub enemy_id: i64,
    pub enemy_name: String,
    pub item_name: String,
    /// Per-kill fraction; for a region resource, the best planet where the enemy appears.
    pub chance: f64,
    /// Region resource: equal share of the planet's resources (rule 14).
    pub estimated: bool,
    /// Compatible nodes within the filters.
    pub spots: i64,
}

/// One node, with the best compatible enemy.
#[derive(Debug, Clone, PartialEq)]
pub struct SpotRow {
    pub node_key: String,
    pub node_name: String,
    pub planet: String,
    pub mission_type: String,
    pub is_endless: bool,
    pub faction: Option<String>,
    pub best_enemy: String,
    /// Per-kill chance of any chosen item on the best enemy.
    pub chance: f64,
    pub estimated: bool,
    /// Compatible enemies dropping a chosen item.
    pub enemies: i64,
}

/// Shared CTEs. `?1` JSON array of item ids, `?2` mission type, `?3` planet.
///
/// - `rates`: per-kill chance of each enemy for each chosen item; a region resource row is
///   tied to its planet and divided by the planet's resource count.
/// - `spot`: compatible (enemy, node) pairs within the filters (rule 13).
/// - `node_enemy`: per-kill chance of any chosen item for an enemy on a node.
const SPOT_CTES: &str = "
    chosen(item_id) AS (SELECT value FROM json_each(?1)),
    region_size AS (SELECT planet_id, count(*) AS n FROM region_resource GROUP BY planet_id),
    rates AS (
        SELECT ed.enemy_id, ed.item_id, NULL AS planet_id, sum(ed.chance) AS chance,
               0 AS estimated
        FROM enemy_drop ed JOIN chosen c ON c.item_id = ed.item_id
        GROUP BY ed.enemy_id, ed.item_id
        UNION ALL
        SELECT ed.enemy_id, rr.item_id, rr.planet_id, sum(ed.chance) / rs.n, 1
        FROM enemy_drop ed
        JOIN item ri ON ri.id = ed.item_id AND ri.name = 'Region Resource'
        JOIN region_resource rr
        JOIN chosen c ON c.item_id = rr.item_id
        JOIN region_size rs ON rs.planet_id = rr.planet_id
        GROUP BY ed.enemy_id, rr.item_id, rr.planet_id
    ),
    spot AS (
        SELECT DISTINCT e.id AS enemy_id, n.id AS node_id, n.planet_id
        FROM enemy e
        JOIN node_faction nf ON nf.faction = e.faction
        JOIN node n ON n.id = nf.node_id
        WHERE e.id IN (SELECT enemy_id FROM rates)
          AND (?2 IS NULL OR n.mission_type_id = ?2)
          AND (?3 IS NULL OR n.planet_id = ?3)
          AND (NOT EXISTS (SELECT 1 FROM enemy_location l
                           WHERE l.enemy_id = e.id AND l.kind = 'planet')
               OR EXISTS (SELECT 1 FROM enemy_location l JOIN planet p ON p.name = l.value
                          WHERE l.enemy_id = e.id AND l.kind = 'planet'
                            AND p.id = n.planet_id))
          AND CASE WHEN EXISTS (SELECT 1 FROM enemy_location l
                                WHERE l.enemy_id = e.id AND l.kind = 'tileset')
                   THEN EXISTS (SELECT 1 FROM enemy_location l
                                WHERE l.enemy_id = e.id AND l.kind = 'tileset'
                                  AND l.value = n.tileset)
                   ELSE ifnull(n.tileset, '') NOT IN ('Free Space', 'Corpus Ship (Archwing)')
              END
    ),
    node_enemy AS (
        SELECT s.node_id, s.enemy_id, sum(r.chance) AS chance, max(r.estimated) AS estimated
        FROM spot s
        JOIN rates r ON r.enemy_id = s.enemy_id
                    AND (r.planet_id IS NULL OR r.planet_id = s.planet_id)
        GROUP BY s.node_id, s.enemy_id
    )";

/// Enemies dropping a chosen item, best chance first. Without a mission type or planet
/// filter, enemies without a known spot are listed too; with one, only enemies with a
/// compatible node.
pub fn enemies_for_items(
    conn: &Connection,
    filter: &SpotFilter,
) -> Result<Vec<EnemyDropRow>, DbError> {
    if filter.item_ids.is_empty() {
        return Ok(Vec::new());
    }
    // A region resource rate counts on the planets where the enemy has a spot; an enemy
    // without any spot keeps its best planet, so that it still shows up unfiltered.
    let sql = format!(
        "WITH {SPOT_CTES},
         enemy_spots AS (SELECT enemy_id, count(DISTINCT node_id) AS spots
                         FROM node_enemy GROUP BY enemy_id)
         SELECT e.id, e.name, i.name, max(r.chance), max(r.estimated), ifnull(es.spots, 0)
         FROM rates r
         JOIN enemy e ON e.id = r.enemy_id
         JOIN item i ON i.id = r.item_id
         LEFT JOIN enemy_spots es ON es.enemy_id = r.enemy_id
         WHERE (r.planet_id IS NULL
                OR r.planet_id IN (SELECT s.planet_id FROM node_enemy ne
                                   JOIN spot s ON s.node_id = ne.node_id
                                    AND s.enemy_id = ne.enemy_id
                                   WHERE ne.enemy_id = r.enemy_id)
                OR es.spots IS NULL)
           AND ((?2 IS NULL AND ?3 IS NULL) OR es.spots IS NOT NULL)
         GROUP BY e.id, i.id
         ORDER BY max(r.chance) DESC, ifnull(es.spots, 0) DESC, e.name, e.key, i.name"
    );
    let mut statement = conn.prepare_cached(&sql)?;
    let rows = statement.query_map(
        params![
            item_ids_json(&filter.item_ids),
            filter.mission_type_id,
            filter.planet_id
        ],
        |row| {
            Ok(EnemyDropRow {
                enemy_id: row.get(0)?,
                enemy_name: row.get(1)?,
                item_name: row.get(2)?,
                chance: row.get(3)?,
                estimated: row.get(4)?,
                spots: row.get(5)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Nodes where a compatible enemy drops a chosen item, ranked by best per-kill chance, number
/// of compatible enemies, endless mission types first (rule 13).
pub fn spots_for_items(conn: &Connection, filter: &SpotFilter) -> Result<Vec<SpotRow>, DbError> {
    if filter.item_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "WITH {SPOT_CTES},
         ranked AS (
             SELECT node_id, enemy_id, chance, estimated,
                    row_number() OVER (PARTITION BY node_id
                                       ORDER BY chance DESC, enemy_id) AS rank,
                    count(*) OVER (PARTITION BY node_id) AS enemies
             FROM node_enemy
             WHERE ?4 IS NULL OR enemy_id = ?4
         )
         SELECT n.key, n.name, p.name, mt.name, mt.is_endless, n.faction, e.name,
                r.chance, r.estimated, r.enemies
         FROM ranked r
         JOIN node n ON n.id = r.node_id
         JOIN planet p ON p.id = n.planet_id
         JOIN mission_type mt ON mt.id = n.mission_type_id
         JOIN enemy e ON e.id = r.enemy_id
         WHERE r.rank = 1
         ORDER BY r.chance DESC, r.enemies DESC, mt.is_endless DESC, p.name, n.name, n.key"
    );
    let mut statement = conn.prepare_cached(&sql)?;
    let rows = statement.query_map(
        params![
            item_ids_json(&filter.item_ids),
            filter.mission_type_id,
            filter.planet_id,
            filter.enemy_id
        ],
        |row| {
            Ok(SpotRow {
                node_key: row.get(0)?,
                node_name: row.get(1)?,
                planet: row.get(2)?,
                mission_type: row.get(3)?,
                is_endless: row.get(4)?,
                faction: row.get(5)?,
                best_enemy: row.get(6)?,
                chance: row.get(7)?,
                estimated: row.get(8)?,
                enemies: row.get(9)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// JSON array of item ids, bound as `?1`. Ids are integers: no escaping needed.
fn item_ids_json(ids: &[i64]) -> String {
    let ids: Vec<String> = ids.iter().map(i64::to_string).collect();
    format!("[{}]", ids.join(","))
}
