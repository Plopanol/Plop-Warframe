//! Items tab queries on the `@wfcd/items` fixtures (`docs/04-REGLES-DROPS.md`, rule 19).

use rusqlite::Connection;

use plop_warframe::db::{self, ImportRun};
use plop_warframe::item_view::detail_table;
use plop_warframe::source::{DropSource, WfcdItemsSource};

#[cfg(test)]
fn clock() -> i64 {
    1_763_500_000
}

#[cfg(test)]
fn database() -> Connection {
    let dataset = WfcdItemsSource::new(
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
    )
    .load()
    .expect("WFCD fixtures");
    let mut conn = db::open_in_memory().expect("open");
    let run = ImportRun {
        source: "test",
        source_revision: None,
    };
    db::import_dataset(&mut conn, &dataset, run, &clock).expect("import");
    conn
}

#[cfg(test)]
fn names(
    conn: &Connection,
    category: Option<&str>,
    query: &str,
    limit: u32,
) -> (Vec<String>, usize) {
    let (hits, total) = db::search_wfcd_items(conn, category, query, limit).expect("search");
    (hits.into_iter().map(|hit| hit.name).collect(), total)
}

#[test]
fn categories_are_those_present() {
    let mut categories = db::wfcd_categories(&database()).expect("categories");
    categories.sort();
    assert_eq!(categories, ["Mods", "Warframes"]);
}

#[test]
fn search_matches_english_and_french_names_without_accents() {
    let conn = database();
    assert_eq!(
        names(&conn, None, "", 10),
        (vec!["Excalibur".into(), "Mag".into(), "Vitality".into()], 3)
    );
    // French name, without case or accents.
    assert_eq!(names(&conn, None, "VITALITE", 10).0, ["Vitality"]);
    assert_eq!(names(&conn, None, "vital", 10).0, ["Vitality"]);
    assert_eq!(names(&conn, None, "cal", 10).0, ["Excalibur"]);
    assert_eq!(names(&conn, Some("Mods"), "", 10).0, ["Vitality"]);
    assert_eq!(names(&conn, Some("Mods"), "mag", 10).1, 0);
}

#[test]
fn total_counts_every_match_beyond_the_limit() {
    let conn = database();
    assert_eq!(
        names(&conn, None, "", 2),
        (vec!["Excalibur".into(), "Mag".into()], 3)
    );
}

#[test]
fn detail_gives_json_and_translation() {
    let conn = database();
    let (hits, _) = db::search_wfcd_items(&conn, None, "Vitality", 1).expect("search");
    let id = hits.first().map(|hit| hit.id).expect("hit");
    let (json, json_fr) = db::wfcd_item_detail(&conn, id)
        .expect("detail")
        .expect("found");
    let table = detail_table(&json, json_fr.as_deref()).expect("detail table");
    assert_eq!(table.rows[0].cells, ["name", "Vitality", "Vitalité"]);
    assert!(db::wfcd_item_detail(&conn, -1).expect("detail").is_none());
}
