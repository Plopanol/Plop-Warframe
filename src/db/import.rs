//! Transactional import: one transaction empties then reloads every data table, so a failed
//! import leaves the previous data intact; `import_run` keeps the history of every attempt.

use std::collections::{BTreeSet, HashMap};

use rusqlite::{Connection, Transaction, params};

use super::DbError;
use crate::model::{DropDataset, node_faction_codes};

/// Placeholder stored when the wiki leaves a planet or mission type empty.
const UNKNOWN: &str = "(inconnu)";

/// Describes where the imported data comes from.
#[derive(Debug, Clone, Copy)]
pub struct ImportRun<'a> {
    /// Short source code, e.g. `wiki` or `wiki-cache`.
    pub source: &'a str,
    /// Revision of the source when known (download dates, `Last-Modified`…).
    pub source_revision: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    pub run_id: i64,
    pub nodes: usize,
    pub tables: usize,
    pub items: usize,
    pub drops: usize,
    pub warnings: usize,
}

/// Replaces the data with `dataset`. `now` returns Unix seconds; it is injected so tests stay
/// deterministic.
pub fn import_dataset(
    conn: &mut Connection,
    dataset: &DropDataset,
    run: ImportRun<'_>,
    now: &dyn Fn() -> i64,
) -> Result<ImportSummary, DbError> {
    conn.execute(
        "INSERT INTO import_run (started_at, source, source_revision, status)
         VALUES (?1, ?2, ?3, 'running')",
        params![now(), run.source, run.source_revision],
    )?;
    let run_id = conn.last_insert_rowid();

    let result = conn
        .transaction()
        .map_err(DbError::from)
        .and_then(|tx| load(tx, dataset, run_id, now));
    if let Err(error) = &result {
        // The data transaction is already rolled back; record why on the run itself.
        conn.execute(
            "UPDATE import_run SET status = 'failed', finished_at = ?1, message = ?2
             WHERE id = ?3",
            params![now(), error.to_string(), run_id],
        )?;
    }
    result
}

fn load(
    tx: Transaction<'_>,
    dataset: &DropDataset,
    run_id: i64,
    now: &dyn Fn() -> i64,
) -> Result<ImportSummary, DbError> {
    // Children first, for the foreign keys. Without AUTOINCREMENT, ids restart at 1, so two
    // imports of the same data give identical rows.
    tx.execute_batch(
        "DELETE FROM region_resource;
         DELETE FROM enemy_drop;
         DELETE FROM enemy_location;
         DELETE FROM enemy;
         DELETE FROM node_faction;
         DELETE FROM \"drop\";
         DELETE FROM node_drop_table;
         DELETE FROM node;
         DELETE FROM item;
         DELETE FROM drop_table;
         DELETE FROM mission_type;
         DELETE FROM planet;",
    )?;

    let planet_ids = insert_planets(&tx, dataset)?;
    let type_ids = insert_mission_types(&tx, dataset)?;
    let mut items = Items::new(&tx)?;
    let (table_ids, drop_count) = insert_tables(&tx, dataset, &mut items)?;

    let mut insert_node = tx.prepare_cached(
        "INSERT INTO node (key, name, planet_id, mission_type_id, faction, min_level, max_level,
                           is_dark_sector, is_special, tileset)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;
    let mut insert_link = tx.prepare_cached(
        "INSERT INTO node_drop_table (node_id, drop_table_id, role) VALUES (?1, ?2, ?3)",
    )?;
    let mut insert_faction =
        tx.prepare_cached("INSERT INTO node_faction (node_id, faction) VALUES (?1, ?2)")?;
    for node in &dataset.nodes {
        insert_node.execute(params![
            node.key,
            node.name,
            planet_ids[or_unknown(&node.planet)],
            type_ids[or_unknown(&node.mission_type)],
            node.faction,
            node.min_level,
            node.max_level,
            node.is_dark_sector,
            node.is_special,
            node.tileset,
        ])?;
        let node_id = tx.last_insert_rowid();
        for link in &node.tables {
            // The source only links existing tables; a missing one is a source bug, skipped.
            if let Some(table_id) = table_ids.get(link.alias.as_str()) {
                insert_link.execute(params![node_id, table_id, link.role.code()])?;
            }
        }
        // Special missions have no fixed enemies.
        if !node.is_special {
            for code in node_faction_codes(node.faction.as_deref().unwrap_or_default()) {
                insert_faction.execute(params![node_id, code])?;
            }
        }
    }
    drop(insert_node);
    drop(insert_link);
    drop(insert_faction);

    insert_enemies(&tx, dataset, &mut items)?;
    insert_region_resources(&tx, dataset, &planet_ids, &mut items)?;
    let item_count = items.count();
    drop(items);

    let summary = ImportSummary {
        run_id,
        nodes: dataset.nodes.len(),
        tables: dataset.tables.len(),
        items: item_count,
        drops: drop_count,
        warnings: dataset.warnings.len(),
    };
    let message = format!(
        "{} nœuds, {} tables, {} objets, {} drops, {} avertissements",
        summary.nodes, summary.tables, summary.items, summary.drops, summary.warnings
    );
    tx.execute(
        "UPDATE import_run SET status = 'success', finished_at = ?1, message = ?2 WHERE id = ?3",
        params![now(), message, run_id],
    )?;
    tx.commit()?;
    Ok(summary)
}

fn or_unknown(value: &str) -> &str {
    if value.trim().is_empty() {
        UNKNOWN
    } else {
        value
    }
}

fn insert_planets<'d>(
    tx: &Transaction<'_>,
    dataset: &'d DropDataset,
) -> Result<HashMap<&'d str, i64>, DbError> {
    // Region planets are usually node planets; the others are kept for completeness.
    let names: BTreeSet<&str> = dataset
        .nodes
        .iter()
        .map(|node| or_unknown(&node.planet))
        .chain(
            dataset
                .region_resources
                .iter()
                .map(|region| or_unknown(&region.planet)),
        )
        .collect();
    let mut statement = tx.prepare_cached("INSERT INTO planet (name) VALUES (?1)")?;
    let mut ids = HashMap::new();
    for name in names {
        statement.execute([name])?;
        ids.insert(name, tx.last_insert_rowid());
    }
    Ok(ids)
}

fn insert_mission_types<'d>(
    tx: &Transaction<'_>,
    dataset: &'d DropDataset,
) -> Result<HashMap<&'d str, i64>, DbError> {
    // Types from `MissionTypes`, plus those only nodes or special tables use (Skirmish +
    // Assassinate, bounty families…), which are not endless.
    let mut types: BTreeSet<(&str, bool)> = dataset
        .mission_types
        .iter()
        .map(|t| (t.name.as_str(), t.is_endless))
        .collect();
    let known: BTreeSet<&str> = types.iter().map(|(name, _)| *name).collect();
    for node in &dataset.nodes {
        let name = or_unknown(&node.mission_type);
        if !known.contains(name) {
            types.insert((name, false));
        }
    }
    let mut statement =
        tx.prepare_cached("INSERT OR IGNORE INTO mission_type (name, is_endless) VALUES (?1, ?2)")?;
    let mut ids = HashMap::new();
    for (name, is_endless) in types {
        statement.execute(params![name, is_endless])?;
        let id = tx.query_row(
            "SELECT id FROM mission_type WHERE name = ?1",
            [name],
            |row| row.get(0),
        )?;
        ids.insert(name, id);
    }
    Ok(ids)
}

type TableIds<'d> = HashMap<&'d str, i64>;

/// Item ids by name. An item keeps the category of its first occurrence; the wiki is not
/// always consistent (an Ayatan sculpture may be "Item" or "Ayatan Sculpture").
struct Items<'t, 'd> {
    insert: rusqlite::CachedStatement<'t>,
    ids: HashMap<&'d str, i64>,
}

impl<'t, 'd> Items<'t, 'd> {
    fn new(tx: &'t Transaction<'_>) -> Result<Self, DbError> {
        Ok(Self {
            insert: tx.prepare_cached("INSERT INTO item (name, category) VALUES (?1, ?2)")?,
            ids: HashMap::new(),
        })
    }

    fn id(&mut self, tx: &Transaction<'_>, name: &'d str, category: &str) -> Result<i64, DbError> {
        if let Some(id) = self.ids.get(name) {
            return Ok(*id);
        }
        self.insert.execute(params![name, category])?;
        let id = tx.last_insert_rowid();
        self.ids.insert(name, id);
        Ok(id)
    }

    fn count(&self) -> usize {
        self.ids.len()
    }
}

fn insert_enemies<'d>(
    tx: &Transaction<'_>,
    dataset: &'d DropDataset,
    items: &mut Items<'_, 'd>,
) -> Result<(), DbError> {
    let mut insert_enemy =
        tx.prepare_cached("INSERT INTO enemy (key, name, faction) VALUES (?1, ?2, ?3)")?;
    let mut insert_location = tx.prepare_cached(
        "INSERT OR IGNORE INTO enemy_location (enemy_id, kind, value) VALUES (?1, ?2, ?3)",
    )?;
    let mut insert_drop = tx.prepare_cached(
        "INSERT INTO enemy_drop (enemy_id, item_id, chance, rarity) VALUES (?1, ?2, ?3, ?4)",
    )?;
    for enemy in &dataset.enemies {
        insert_enemy.execute(params![enemy.key, enemy.name, enemy.faction])?;
        let enemy_id = tx.last_insert_rowid();
        let locations = enemy
            .planets
            .iter()
            .map(|planet| ("planet", planet))
            .chain(enemy.tilesets.iter().map(|tileset| ("tileset", tileset)));
        for (kind, value) in locations {
            insert_location.execute(params![enemy_id, kind, value])?;
        }
        for drop in &enemy.drops {
            let item_id = items.id(tx, &drop.item, &drop.category)?;
            insert_drop.execute(params![enemy_id, item_id, drop.chance, drop.rarity.code()])?;
        }
    }
    Ok(())
}

fn insert_region_resources<'d>(
    tx: &Transaction<'_>,
    dataset: &'d DropDataset,
    planet_ids: &HashMap<&str, i64>,
    items: &mut Items<'_, 'd>,
) -> Result<(), DbError> {
    let mut insert = tx.prepare_cached(
        "INSERT OR IGNORE INTO region_resource (planet_id, item_id) VALUES (?1, ?2)",
    )?;
    for region in &dataset.region_resources {
        let Some(planet_id) = planet_ids.get(or_unknown(&region.planet)) else {
            continue;
        };
        for resource in &region.resources {
            let item_id = items.id(tx, resource, "Resource")?;
            insert.execute(params![planet_id, item_id])?;
        }
    }
    Ok(())
}

fn insert_tables<'d>(
    tx: &Transaction<'_>,
    dataset: &'d DropDataset,
    items: &mut Items<'_, 'd>,
) -> Result<(TableIds<'d>, usize), DbError> {
    let mut insert_table = tx.prepare_cached(
        "INSERT INTO drop_table (alias, name, table_type, tier, rotation_kind)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    let mut insert_drop = tx.prepare_cached(
        "INSERT INTO \"drop\" (drop_table_id, rotation, item_id, chance, rarity, quantity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;

    let mut table_ids = HashMap::new();
    let mut drop_count = 0;
    for table in &dataset.tables {
        insert_table.execute(params![
            table.alias,
            table.name,
            table.table_type,
            table.tier,
            table.kind.code()
        ])?;
        let table_id = tx.last_insert_rowid();
        table_ids.insert(table.alias.as_str(), table_id);

        for entry in &table.entries {
            let item_id = items.id(tx, &entry.item, &entry.category)?;
            insert_drop.execute(params![
                table_id,
                entry.rotation.code(),
                item_id,
                entry.chance,
                entry.rarity.code(),
                entry.quantity
            ])?;
            drop_count += 1;
        }
    }
    Ok((table_ids, drop_count))
}
