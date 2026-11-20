//! Queries of the Relics tab: Warframes found in the relic rewards and the relics giving
//! their parts. Rules: `docs/04-REGLES-DROPS.md` (rules 16 and 17).

use rusqlite::{Connection, params};

use super::DbError;
use crate::model::Refinement;

/// Parts of a Warframe, in display order; each name is `<Warframe><suffix>`.
pub const WARFRAME_PARTS: [&str; 4] = [
    " Prime Blueprint",
    " Prime Neuroptics Blueprint",
    " Prime Chassis Blueprint",
    " Prime Systems Blueprint",
];

/// Warframes: names given both a Prime Neuroptics and a Prime Chassis blueprint by the relics,
/// which leaves out sentinels, Archwings and weapons.
pub fn warframes(conn: &Connection) -> Result<Vec<String>, DbError> {
    let mut statement = conn.prepare_cached(
        "SELECT substr(item_name, 1, length(item_name) - length(?1)) AS name
         FROM relic_reward WHERE item_name LIKE '%' || ?1
         INTERSECT
         SELECT substr(item_name, 1, length(item_name) - length(?2))
         FROM relic_reward WHERE item_name LIKE '%' || ?2
         ORDER BY name COLLATE NOCASE, name",
    )?;
    let rows = statement.query_map(params![WARFRAME_PARTS[1], WARFRAME_PARTS[2]], |row| {
        row.get(0)
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// One relic giving one part, with its chance at each refinement.
#[derive(Debug, Clone, PartialEq)]
pub struct RelicRow {
    pub part: String,
    pub relic: String,
    /// Chance by refinement, in [`Refinement::ALL`] order; `None` when DE does not list it.
    pub chances: [Option<f64>; 4],
    /// The relic (name without « Relic ») is an item of an imported mission drop table.
    pub in_mission: bool,
}

/// Relics giving the parts of `warframe`, by part order then relic name.
pub fn relics_for_warframe(conn: &Connection, warframe: &str) -> Result<Vec<RelicRow>, DbError> {
    let [intact, exceptional, flawless, radiant] = Refinement::ALL.map(Refinement::code);
    let mut statement = conn.prepare_cached(
        "SELECT rr.item_name, r.name,
                max(CASE WHEN rr.refinement = ?6 THEN rr.chance END),
                max(CASE WHEN rr.refinement = ?7 THEN rr.chance END),
                max(CASE WHEN rr.refinement = ?8 THEN rr.chance END),
                max(CASE WHEN rr.refinement = ?9 THEN rr.chance END),
                EXISTS (
                    SELECT 1 FROM item i JOIN \"drop\" d ON d.item_id = i.id
                    WHERE i.name = CASE WHEN r.name LIKE '% Relic'
                                        THEN substr(r.name, 1, length(r.name) - 6)
                                        ELSE r.name END
                )
         FROM relic_reward rr JOIN relic r ON r.id = rr.relic_id
         WHERE rr.item_name IN (?1 || ?2, ?1 || ?3, ?1 || ?4, ?1 || ?5)
         GROUP BY rr.item_name, r.id
         ORDER BY CASE rr.item_name WHEN ?1 || ?2 THEN 0 WHEN ?1 || ?3 THEN 1
                                    WHEN ?1 || ?4 THEN 2 ELSE 3 END,
                  r.name COLLATE NOCASE",
    )?;
    let rows = statement.query_map(
        params![
            warframe,
            WARFRAME_PARTS[0],
            WARFRAME_PARTS[1],
            WARFRAME_PARTS[2],
            WARFRAME_PARTS[3],
            intact,
            exceptional,
            flawless,
            radiant
        ],
        |row| {
            Ok(RelicRow {
                part: row.get(0)?,
                relic: row.get(1)?,
                chances: [row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?],
                in_mission: row.get(6)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}
