//! Transactional import: one transaction empties then reloads every data table, so a failed
//! import leaves the previous data intact; `import_run` keeps the history of every attempt.

use std::collections::{BTreeSet, HashMap};

use rusqlite::{Connection, Transaction, params};

use super::DbError;
use crate::model::DropDataset;

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
        "DELETE FROM \"drop\";
         DELETE FROM node_drop_table;
         DELETE FROM node;
         DELETE FROM item;
         DELETE FROM drop_table;
         DELETE FROM mission_type;
         DELETE FROM planet;",
    )?;

    let planet_ids = insert_planets(&tx, dataset)?;
    let type_ids = insert_mission_types(&tx, dataset)?;
    let (table_ids, item_count, drop_count) = insert_tables(&tx, dataset)?;

    let mut insert_node = tx.prepare_cached(
        "INSERT INTO node (key, name, planet_id, mission_type_id, faction, min_level, max_level,
                           is_dark_sector, is_special)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    let mut insert_link = tx.prepare_cached(
        "INSERT INTO node_drop_table (node_id, drop_table_id, role) VALUES (?1, ?2, ?3)",
    )?;
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
        ])?;
        let node_id = tx.last_insert_rowid();
        for link in &node.tables {
            // The source only links existing tables; a missing one is a source bug, skipped.
            if let Some(table_id) = table_ids.get(link.alias.as_str()) {
                insert_link.execute(params![node_id, table_id, link.role.code()])?;
            }
        }
    }
    drop(insert_node);
    drop(insert_link);

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
    let names: BTreeSet<&str> = dataset
        .nodes
        .iter()
        .map(|node| or_unknown(&node.planet))
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

fn insert_tables<'d>(
    tx: &Transaction<'_>,
    dataset: &'d DropDataset,
) -> Result<(TableIds<'d>, usize, usize), DbError> {
    let mut insert_table = tx.prepare_cached(
        "INSERT INTO drop_table (alias, name, table_type, tier, rotation_kind)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    let mut insert_item = tx.prepare_cached("INSERT INTO item (name, category) VALUES (?1, ?2)")?;
    let mut insert_drop = tx.prepare_cached(
        "INSERT INTO \"drop\" (drop_table_id, rotation, item_id, chance, rarity, quantity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;

    let mut table_ids = HashMap::new();
    // An item keeps the category of its first occurrence; the wiki is not always consistent
    // (an Ayatan sculpture may be "Item" or "Ayatan Sculpture").
    let mut item_ids: HashMap<&str, i64> = HashMap::new();
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
            let item_id = match item_ids.get(entry.item.as_str()) {
                Some(id) => *id,
                None => {
                    insert_item.execute(params![entry.item, entry.category])?;
                    let id = tx.last_insert_rowid();
                    item_ids.insert(entry.item.as_str(), id);
                    id
                }
            };
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
    Ok((table_ids, item_ids.len(), drop_count))
}
