//! Import tests: full fixtures into an in-memory database.

use std::sync::OnceLock;

use plop_warframe::db::{self, ImportRun};
use plop_warframe::model::{DropDataset, DropEntry, DropTable, Rarity, Rotation, RotationKind};
use plop_warframe::source::{DropSource, WikiSource};
use rusqlite::Connection;

const RUN: ImportRun<'static> = ImportRun {
    source: "test",
    source_revision: Some("fixtures 2026-11-19"),
};

#[cfg(test)]
fn dataset() -> &'static DropDataset {
    static DATASET: OnceLock<DropDataset> = OnceLock::new();
    DATASET.get_or_init(|| {
        WikiSource::new(
            include_str!("fixtures/droptables_data.lua"),
            include_str!("fixtures/missions_data.lua"),
        )
        .load()
        .expect("fixtures must parse")
    })
}

#[cfg(test)]
fn clock() -> i64 {
    1_763_500_000
}

/// Every data row, in a stable order, to compare two imports.
#[cfg(test)]
fn dump(conn: &Connection) -> Vec<String> {
    let queries = [
        "SELECT id || '|' || name FROM planet ORDER BY id",
        "SELECT id || '|' || name || '|' || is_endless FROM mission_type ORDER BY id",
        "SELECT id || '|' || key || '|' || planet_id || '|' || mission_type_id || '|' ||
                ifnull(faction, '') || '|' || is_special FROM node ORDER BY id",
        "SELECT id || '|' || alias || '|' || rotation_kind FROM drop_table ORDER BY id",
        "SELECT node_id || '|' || drop_table_id || '|' || role FROM node_drop_table
         ORDER BY node_id, drop_table_id, role",
        "SELECT id || '|' || name || '|' || category FROM item ORDER BY id",
        "SELECT id || '|' || drop_table_id || '|' || rotation || '|' || item_id || '|' ||
                chance || '|' || rarity || '|' || quantity FROM \"drop\" ORDER BY id",
    ];
    let mut rows = Vec::new();
    for sql in queries {
        let mut statement = conn.prepare(sql).expect("prepare");
        let values = statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query");
        rows.extend(values.map(|value| value.expect("row")));
    }
    rows
}

#[test]
fn migrations_set_the_schema_version() {
    let conn = db::open_in_memory().expect("open");
    assert_eq!(db::schema_version(&conn).expect("version"), 4);
    let foreign_keys: i64 = conn
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .expect("pragma");
    assert_eq!(foreign_keys, 1);
}

#[test]
fn reopening_a_file_keeps_data_and_version() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(db::DATABASE_FILE);
    {
        let mut conn = db::open(&path).expect("open");
        db::import_dataset(&mut conn, dataset(), RUN, &clock).expect("import");
    }
    let conn = db::open(&path).expect("reopen");
    assert_eq!(db::schema_version(&conn).expect("version"), 4);
    assert_eq!(db::stats(&conn).expect("stats").nodes, 524);
}

#[test]
fn newer_schema_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(db::DATABASE_FILE);
    {
        let conn = Connection::open(&path).expect("raw open");
        conn.pragma_update(None, "user_version", 99)
            .expect("pragma");
    }
    assert!(matches!(
        db::open(&path),
        Err(db::DbError::TooNew { found: 99, .. })
    ));
}

#[test]
fn full_import_fills_the_tables() {
    let mut conn = db::open_in_memory().expect("open");
    let summary = db::import_dataset(&mut conn, dataset(), RUN, &clock).expect("import");
    assert_eq!(summary.nodes, 524);
    assert_eq!(summary.tables, 341);
    assert_eq!(summary.drops, 6753);

    let stats = db::stats(&conn).expect("stats");
    assert_eq!(stats.nodes, 524);
    assert_eq!(stats.drops, 6753);
    assert_eq!(usize::try_from(stats.items).expect("items"), summary.items);
    let run = stats.last_run.expect("run");
    assert_eq!(run.status, "success");
    assert_eq!(run.finished_at, Some(clock()));
    assert_eq!(run.source_revision.as_deref(), Some("fixtures 2026-11-19"));

    // Tier 1 Survival, rotation C, Lith L8 at 11.06 %, reached through Apollodorus.
    let chance: f64 = conn
        .query_row(
            "SELECT d.chance FROM node n
             JOIN node_drop_table l ON l.node_id = n.id
             JOIN drop_table t ON t.id = l.drop_table_id
             JOIN \"drop\" d ON d.drop_table_id = t.id
             JOIN item i ON i.id = d.item_id
             WHERE n.name = 'Apollodorus' AND d.rotation = 'C' AND i.name = 'Lith L8'",
            [],
            |row| row.get(0),
        )
        .expect("Lith L8");
    assert!((chance - 0.1106).abs() < 1e-9);

    let survival_endless: bool = conn
        .query_row(
            "SELECT is_endless FROM mission_type WHERE name = 'Survival'",
            [],
            |row| row.get(0),
        )
        .expect("Survival");
    assert!(survival_endless);
}

#[test]
fn second_import_gives_identical_data() {
    let mut conn = db::open_in_memory().expect("open");
    db::import_dataset(&mut conn, dataset(), RUN, &clock).expect("first import");
    let first = dump(&conn);
    db::import_dataset(&mut conn, dataset(), RUN, &clock).expect("second import");
    assert_eq!(dump(&conn), first);

    let runs: i64 = conn
        .query_row("SELECT count(*) FROM import_run", [], |row| row.get(0))
        .expect("runs");
    assert_eq!(runs, 2);
}

#[test]
fn failed_import_keeps_previous_data() {
    let mut conn = db::open_in_memory().expect("open");
    db::import_dataset(&mut conn, dataset(), RUN, &clock).expect("first import");
    let before = dump(&conn);

    // A chance above 1 violates the CHECK constraint halfway through the load.
    let mut broken = dataset().clone();
    broken.tables.push(DropTable {
        alias: "Broken".to_owned(),
        name: "Broken".to_owned(),
        table_type: "Survival".to_owned(),
        tier: None,
        kind: RotationKind::None,
        entries: vec![DropEntry {
            rotation: Rotation::None,
            item: "Nothing".to_owned(),
            category: "Item".to_owned(),
            chance: 2.0,
            quantity: 1,
            rarity: Rarity::VeryCommon,
        }],
    });
    assert!(db::import_dataset(&mut conn, &broken, RUN, &clock).is_err());

    assert_eq!(dump(&conn), before);
    let last = db::stats(&conn).expect("stats").last_run.expect("run");
    assert_eq!(last.status, "failed");
    assert!(last.message.is_some());
}
