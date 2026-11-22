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
    assert_eq!(db::schema_version(&conn).expect("version"), 5);
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
    assert_eq!(db::schema_version(&conn).expect("version"), 5);
    assert_eq!(db::stats(&conn).expect("stats").nodes, 523);
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
    assert_eq!(summary.nodes, 523);
    assert_eq!(summary.tables, 341);
    assert_eq!(summary.drops, 6753);

    let stats = db::stats(&conn).expect("stats");
    assert_eq!(stats.nodes, 523);
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
fn blind_rage_is_found_in_deimos_vaults_without_special_missions() {
    use plop_warframe::db::{NodeFilter, SearchFilter};
    use plop_warframe::model::TableRole;

    let mut conn = db::open_in_memory().expect("open");
    db::import_dataset(&mut conn, dataset(), RUN, &clock).expect("import");
    let item_id = conn
        .query_row("SELECT id FROM item WHERE name = 'Blind Rage'", [], |row| {
            row.get(0)
        })
        .expect("Blind Rage");
    let filter = SearchFilter {
        item_ids: vec![item_id],
        specials: NodeFilter::Exclude,
        ..SearchFilter::default()
    };
    let rows = db::search_by_item(&conn, &filter).expect("search");
    let mut names: Vec<_> = rows
        .iter()
        .map(|row| row.location.node_name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        ["Dirus", "Formido", "Horend", "Phlegyas", "Terrorem"]
    );
    for row in &rows {
        assert_eq!(row.location.planet, "Deimos");
        assert_eq!(row.location.table_alias, "DerelictVault");
        assert_eq!(row.location.role, TableRole::Extra);
        assert_eq!(row.location.rotation_kind, RotationKind::None);
        assert_eq!(row.location.rotation, Rotation::None);
        assert!((row.chance - 0.0417).abs() < 1e-9);
        assert_eq!(row.aabc, None);
    }
    let combined = db::search_combined(&conn, &filter).expect("combined search");
    assert_eq!(combined.len(), 5);
    assert!(
        combined
            .iter()
            .all(|row| (row.chance - 0.0417).abs() < 1e-9)
    );

    let capture = conn
        .query_row(
            "SELECT id FROM mission_type WHERE name = 'Capture'",
            [],
            |row| row.get(0),
        )
        .expect("Capture");
    let captures = db::search_by_item(
        &conn,
        &SearchFilter {
            mission_type_id: Some(capture),
            ..filter
        },
    )
    .expect("capture search");
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].location.node_name, "Horend");
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

#[test]
fn wfcd_items_are_stored_with_counts_and_french_name() {
    use plop_warframe::source::WfcdItemsSource;
    let source = WfcdItemsSource::new(
        vec![
            (
                "Warframes".to_owned(),
                include_str!("fixtures/wfcd/Warframes.json").to_owned(),
            ),
            (
                "Mods".to_owned(),
                include_str!("fixtures/wfcd/Mods.json").to_owned(),
            ),
        ],
        Some(include_str!("fixtures/wfcd/fr.json").to_owned()),
    );
    let dataset = source.load().expect("load");
    assert_eq!(dataset.warnings.len(), 1);
    let mut conn = db::open_in_memory().expect("open");
    let summary = db::import_dataset(&mut conn, &dataset, RUN, &clock).expect("import");
    assert_eq!(
        summary.wfcd_items_by_category,
        [("Warframes".to_owned(), 2), ("Mods".to_owned(), 1)]
    );
    let rows: Vec<(String, Option<String>, bool)> = conn
        .prepare("SELECT name, name_fr, json_fr IS NOT NULL FROM wfcd_item ORDER BY id")
        .expect("prepare")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    assert_eq!(
        rows,
        [
            ("Excalibur".to_owned(), None, false),
            ("Mag".to_owned(), None, false),
            ("Vitality".to_owned(), Some("Vitalité".to_owned()), true),
        ]
    );

    // A second import replaces the items instead of adding to them.
    let summary = db::import_dataset(&mut conn, &dataset, RUN, &clock).expect("import");
    assert_eq!(summary.wfcd_items(), 3);
    let count: i64 = conn
        .query_row("SELECT count(*) FROM wfcd_item", [], |row| row.get(0))
        .expect("count");
    assert_eq!(count, 3);
}
