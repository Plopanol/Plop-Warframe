//! Search query tests on a small hand-made database, plus a timing on the full development
//! database (ignored by default).

use std::path::Path;
use std::time::{Duration, Instant};

use plop_warframe::db::{
    self, CombinedRow, ImportRun, ItemDropRow, RotationFilter, SearchFilter, SpecialFilter,
};
use plop_warframe::model::{
    DropDataset, DropEntry, DropTable, MissionType, Node, NodeTable, Rarity, Rotation,
    RotationKind, TableRole,
};
use rusqlite::Connection;

const EPSILON: f64 = 1e-9;

fn entry(rotation: Rotation, item: &str, chance: f64, quantity: u32) -> DropEntry {
    DropEntry {
        rotation,
        item: item.to_owned(),
        category: "Relic".to_owned(),
        chance,
        quantity,
        rarity: Rarity::from_chance(chance),
    }
}

fn table(alias: &str, table_type: &str, kind: RotationKind, entries: Vec<DropEntry>) -> DropTable {
    DropTable {
        alias: alias.to_owned(),
        name: alias.to_owned(),
        table_type: table_type.to_owned(),
        tier: None,
        kind,
        entries,
    }
}

fn node(key: &str, mission_type: &str, is_special: bool, tables: &[(&str, TableRole)]) -> Node {
    Node {
        key: key.to_owned(),
        name: key.to_owned(),
        planet: "Void".to_owned(),
        mission_type: mission_type.to_owned(),
        faction: None,
        min_level: None,
        max_level: None,
        is_dark_sector: false,
        is_special,
        tileset: None,
        tables: tables
            .iter()
            .map(|(alias, role)| NodeTable {
                alias: (*alias).to_owned(),
                role: *role,
            })
            .collect(),
    }
}

/// Survival (endless, rotations) shared by two nodes, Capture with a single pool and resource
/// caches, a special bounty with stages.
#[cfg(test)]
fn database() -> Connection {
    use Rotation::{A, B, C, None as Single};
    let dataset = DropDataset {
        mission_types: vec![
            MissionType {
                name: "Survival".to_owned(),
                is_endless: true,
            },
            MissionType {
                name: "Capture".to_owned(),
                is_endless: false,
            },
        ],
        nodes: vec![
            node("Mot", "Survival", false, &[("Surv", TableRole::Main)]),
            node("Zeta", "Survival", false, &[("Surv", TableRole::Main)]),
            node(
                "Cap",
                "Capture",
                false,
                &[("Capt", TableRole::Main), ("Caches", TableRole::Cache)],
            ),
            node(
                "special:Bounty",
                "Cetus Bounty",
                true,
                &[("Bounty", TableRole::Main)],
            ),
        ],
        tables: vec![
            table(
                "Surv",
                "Survival",
                RotationKind::Rotation,
                vec![
                    entry(A, "Relic X", 0.10, 1),
                    entry(A, "Höllars Cache", 0.05, 1),
                    entry(B, "Relic X", 0.20, 1),
                    entry(B, "Relic Y", 0.30, 1),
                    entry(C, "Relic X", 0.40, 1),
                    entry(C, "Relic X", 0.10, 2),
                    entry(C, "Relic Y", 0.20, 1),
                ],
            ),
            table(
                "Capt",
                "Capture",
                RotationKind::None,
                vec![
                    entry(Single, "Relic X", 0.50, 1),
                    entry(Single, "Relic Y", 0.25, 1),
                    entry(Single, "Credits 50%", 0.25, 1),
                ],
            ),
            table(
                "Caches",
                "Capture",
                RotationKind::Cache,
                vec![entry(A, "Relic X", 0.30, 1), entry(B, "Relic Y", 0.60, 1)],
            ),
            table(
                "Bounty",
                "Cetus Bounty",
                RotationKind::Stage,
                vec![entry(A, "Relic X", 0.15, 1), entry(C, "Relic Y", 0.35, 1)],
            ),
        ],
        warnings: Vec::new(),
        ..DropDataset::default()
    };
    let mut conn = db::open_in_memory().expect("open");
    db::import_dataset(
        &mut conn,
        &dataset,
        ImportRun {
            source: "test",
            source_revision: None,
        },
        &|| 0,
    )
    .expect("import");
    conn
}

#[cfg(test)]
fn item_id(conn: &Connection, name: &str) -> i64 {
    conn.query_row("SELECT id FROM item WHERE name = ?1", [name], |row| {
        row.get(0)
    })
    .expect("item")
}

#[cfg(test)]
fn mission_type_id(conn: &Connection, name: &str) -> i64 {
    conn.query_row(
        "SELECT id FROM mission_type WHERE name = ?1",
        [name],
        |row| row.get(0),
    )
    .expect("mission type")
}

fn filter(conn: &Connection, items: &[&str]) -> SearchFilter {
    SearchFilter {
        item_ids: items.iter().map(|name| item_id(conn, name)).collect(),
        ..SearchFilter::default()
    }
}

/// `node|table|rotation|quantity` → (chance, aabc) for "by item" rows of one item.
fn by_item_keys(rows: &[ItemDropRow]) -> Vec<(String, f64, Option<f64>)> {
    rows.iter()
        .map(|row| {
            (
                format!(
                    "{}|{}|{}|{}",
                    row.location.node_key,
                    row.location.table_alias,
                    row.location.rotation.code(),
                    row.quantity
                ),
                row.chance,
                row.aabc,
            )
        })
        .collect()
}

#[cfg(test)]
fn combined_row<'r>(rows: &'r [CombinedRow], node: &str, rotation: Rotation) -> &'r CombinedRow {
    rows.iter()
        .find(|row| row.location.node_key == node && row.location.rotation == rotation)
        .unwrap_or_else(|| panic!("no row for {node} {rotation:?}"))
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < EPSILON
}

#[test]
fn fold_text_ignores_case_and_accents() {
    assert_eq!(db::fold_text("Höllars CACHE"), "hollars cache");
    assert_eq!(
        db::fold_text("Œuvre Ærø Straße Ōtak"),
        "oeuvre aero strasse otak"
    );
}

#[test]
fn item_search_contains_ignoring_case_and_accents() {
    let conn = database();
    let names = |text: &str| -> Vec<String> {
        db::search_items(&conn, text, 10)
            .expect("search")
            .into_iter()
            .map(|hit| hit.name)
            .collect()
    };
    assert_eq!(names("HOLL"), ["Höllars Cache"]);
    assert_eq!(names("öll"), ["Höllars Cache"]);
    // Prefix matches first, then by name.
    assert_eq!(names("re"), ["Relic X", "Relic Y", "Credits 50%"]);
    assert_eq!(names("relic"), ["Relic X", "Relic Y"]);
    // SQL wildcards are literal.
    assert_eq!(names("%"), ["Credits 50%"]);
    assert_eq!(names("_"), Vec::<String>::new());
    assert!(names("  ").is_empty());
    assert_eq!(db::search_items(&conn, "r", 1).expect("search").len(), 1);
}

#[test]
fn mission_types_count_their_nodes() {
    let conn = database();
    let types = db::mission_types(&conn).expect("types");
    let summary: Vec<(String, bool, i64, i64)> = types
        .into_iter()
        .map(|t| (t.name, t.is_endless, t.nodes, t.special_nodes))
        .collect();
    assert_eq!(
        summary,
        [
            ("Capture".to_owned(), false, 1, 0),
            ("Cetus Bounty".to_owned(), false, 1, 1),
            ("Survival".to_owned(), true, 2, 0),
        ]
    );
}

#[test]
fn by_item_lists_every_entry_with_its_aabc_chance() {
    let conn = database();
    let rows = db::search_by_item(&conn, &filter(&conn, &["Relic X"])).expect("search");
    let mut keys = by_item_keys(&rows);
    keys.sort_by(|a, b| a.0.cmp(&b.0));
    let aabc = Some((2.0 * 0.10 + 0.20 + 0.50) / 4.0);
    let expected: Vec<(&str, f64, Option<f64>)> = vec![
        ("Cap|Caches|A|1", 0.30, None),
        ("Cap|Capt|NONE|1", 0.50, None),
        ("Mot|Surv|A|1", 0.10, aabc),
        ("Mot|Surv|B|1", 0.20, aabc),
        ("Mot|Surv|C|1", 0.40, aabc),
        ("Mot|Surv|C|2", 0.10, aabc),
        ("Zeta|Surv|A|1", 0.10, aabc),
        ("Zeta|Surv|B|1", 0.20, aabc),
        ("Zeta|Surv|C|1", 0.40, aabc),
        ("Zeta|Surv|C|2", 0.10, aabc),
        ("special:Bounty|Bounty|A|1", 0.15, None),
    ];
    assert_eq!(keys.len(), expected.len(), "{keys:?}");
    for ((key, chance, aabc), (expected_key, expected_chance, expected_aabc)) in
        keys.iter().zip(&expected)
    {
        assert_eq!(key, expected_key);
        assert!(close(*chance, *expected_chance), "{key}: {chance}");
        match (aabc, expected_aabc) {
            (Some(actual), Some(expected)) => assert!(close(*actual, *expected), "{key}"),
            (None, None) => {}
            _ => panic!("{key}: AABC {aabc:?}, expected {expected_aabc:?}"),
        }
    }

    // Sorted by decreasing chance; details of the best row.
    assert!(rows.windows(2).all(|w| w[0].chance >= w[1].chance));
    let best = &rows[0];
    assert_eq!(best.location.node_key, "Cap");
    assert_eq!(best.location.mission_type, "Capture");
    assert_eq!(best.location.rotation_kind, RotationKind::None);
    assert_eq!(best.location.role, TableRole::Main);
    assert_eq!(best.item_name, "Relic X");
    assert_eq!(best.rarity, Rarity::Common);
}

#[test]
fn rotation_filter_only_applies_to_rotation_tables() {
    let conn = database();
    let search = SearchFilter {
        rotations: RotationFilter {
            a: false,
            b: false,
            c: true,
        },
        ..filter(&conn, &["Relic X"])
    };
    let rows = db::search_by_item(&conn, &search).expect("search");
    let mut keys: Vec<String> = by_item_keys(&rows).into_iter().map(|k| k.0).collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "Cap|Caches|A|1",
            "Cap|Capt|NONE|1",
            "Mot|Surv|C|1",
            "Mot|Surv|C|2",
            "Zeta|Surv|C|1",
            "Zeta|Surv|C|2",
            "special:Bounty|Bounty|A|1",
        ]
    );
    // The AABC chance still uses every rotation.
    let surv = rows
        .iter()
        .find(|row| row.location.table_alias == "Surv")
        .expect("Surv row");
    assert!(close(surv.aabc.expect("aabc"), 0.225));
}

#[test]
fn special_and_mission_type_filters() {
    let conn = database();
    let nodes = |search: &SearchFilter| -> Vec<String> {
        let mut keys: Vec<String> = db::search_combined(&conn, search)
            .expect("search")
            .into_iter()
            .map(|row| row.location.node_key)
            .collect();
        keys.sort();
        keys.dedup();
        keys
    };
    let base = filter(&conn, &["Relic X"]);
    assert_eq!(nodes(&base), ["Cap", "Mot", "Zeta", "special:Bounty"]);
    let exclude = SearchFilter {
        specials: SpecialFilter::Exclude,
        ..base.clone()
    };
    assert_eq!(nodes(&exclude), ["Cap", "Mot", "Zeta"]);
    let only = SearchFilter {
        specials: SpecialFilter::Only,
        ..base.clone()
    };
    assert_eq!(nodes(&only), ["special:Bounty"]);
    let survival = SearchFilter {
        mission_type_id: Some(mission_type_id(&conn, "Survival")),
        ..base
    };
    assert_eq!(nodes(&survival), ["Mot", "Zeta"]);
}

#[test]
fn combined_sums_the_chosen_items_per_node_and_rotation() {
    let conn = database();
    let rows = db::search_combined(&conn, &filter(&conn, &["Relic X", "Relic Y"])).expect("search");
    // 2 Survival nodes × 3 rotations + Capture single pool + 2 caches + 2 bounty stages.
    assert_eq!(rows.len(), 11);
    assert!(rows.windows(2).all(|w| w[0].chance >= w[1].chance));

    let check = |node: &str, rotation: Rotation, chance: f64, items: u32, aabc: Option<f64>| {
        let row = combined_row(&rows, node, rotation);
        assert!(
            close(row.chance, chance),
            "{node} {rotation:?}: {}",
            row.chance
        );
        assert_eq!(row.items, items, "{node} {rotation:?}");
        match (row.aabc, aabc) {
            (Some(actual), Some(expected)) => assert!(close(actual, expected)),
            (None, None) => {}
            (actual, _) => panic!("{node} {rotation:?}: AABC {actual:?}"),
        }
    };
    // A: Relic X only (Höllars Cache not chosen); C: 0.40 + 0.10 (quantity 2) + 0.20.
    let aabc = Some((2.0 * 0.10 + 0.50 + 0.70) / 4.0);
    check("Mot", Rotation::A, 0.10, 1, aabc);
    check("Mot", Rotation::B, 0.50, 2, aabc);
    check("Mot", Rotation::C, 0.70, 2, aabc);
    check("Zeta", Rotation::C, 0.70, 2, aabc);
    check("Cap", Rotation::None, 0.75, 2, None);
    let caches: Vec<&CombinedRow> = rows
        .iter()
        .filter(|row| row.location.table_alias == "Caches")
        .collect();
    assert_eq!(caches.len(), 2);
    assert!(
        caches
            .iter()
            .all(|row| row.location.role == TableRole::Cache)
    );
    check("special:Bounty", Rotation::C, 0.35, 1, None);
}

#[test]
fn no_chosen_item_gives_no_result() {
    let conn = database();
    let search = SearchFilter::default();
    assert!(
        db::search_by_item(&conn, &search)
            .expect("search")
            .is_empty()
    );
    assert!(
        db::search_combined(&conn, &search)
            .expect("search")
            .is_empty()
    );
}

/// Timing on the full development database (`data/`, see TASK-009); skipped without it.
/// `cargo test --release --test query_tests -- --ignored --nocapture`
#[test]
#[ignore = "needs the imported development database in data/"]
fn full_database_search_under_100_ms() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/plop-warframe.sqlite3");
    if !path.exists() {
        eprintln!("{} absent : mesure non exécutée", path.display());
        return;
    }
    let conn = db::open(&path).expect("open");
    let ids = |sql: &str| -> Vec<i64> {
        let mut statement = conn.prepare(sql).expect("prepare");
        statement
            .query_map([], |row| row.get(0))
            .expect("query")
            .map(|id| id.expect("id"))
            .collect()
    };
    let frequent =
        ids("SELECT item_id FROM \"drop\" GROUP BY item_id ORDER BY count(*) DESC LIMIT 20");
    let every = ids("SELECT id FROM item");
    let limit = Duration::from_millis(100);

    let time = |label: &str, run: &dyn Fn() -> usize| {
        run(); // Warm the statement cache and the page cache.
        let mut worst = Duration::ZERO;
        let mut rows = 0;
        for _ in 0..5 {
            let start = Instant::now();
            rows = run();
            worst = worst.max(start.elapsed());
        }
        println!("{label}: {rows} lignes, pire de 5 : {worst:?}");
        assert!(worst < limit, "{label}: {worst:?}");
    };
    let all = |item_ids: &[i64]| SearchFilter {
        item_ids: item_ids.to_vec(),
        ..SearchFilter::default()
    };
    time("autocomplétion « a »", &|| {
        db::search_items(&conn, "a", 50).expect("search").len()
    });
    time("types de mission", &|| {
        db::mission_types(&conn).expect("types").len()
    });
    time("par objet, 20 objets fréquents", &|| {
        db::search_by_item(&conn, &all(&frequent))
            .expect("search")
            .len()
    });
    time("combiné, 20 objets fréquents", &|| {
        db::search_combined(&conn, &all(&frequent))
            .expect("search")
            .len()
    });
    time("par objet, tous les objets", &|| {
        db::search_by_item(&conn, &all(&every))
            .expect("search")
            .len()
    });
    time("combiné, tous les objets", &|| {
        db::search_combined(&conn, &all(&every))
            .expect("search")
            .len()
    });
}
