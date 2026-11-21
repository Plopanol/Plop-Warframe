//! Star Chart queries on a small hand-made dataset (rule 16 of `docs/04-REGLES-DROPS.md`).

use plop_warframe::db::{self, ImportRun, NodeFilter, StarChartFilter};
use plop_warframe::model::{DropDataset, MissionType, Node};
use rusqlite::Connection;

#[cfg(test)]
fn node(
    name: &str,
    planet: &str,
    mission_type: &str,
    faction: &str,
    levels: Option<(u32, u32)>,
) -> Node {
    Node {
        key: name.to_owned(),
        name: name.to_owned(),
        planet: planet.to_owned(),
        mission_type: mission_type.to_owned(),
        faction: Some(faction.to_owned()),
        min_level: levels.map(|(min, _)| min),
        max_level: levels.map(|(_, max)| max),
        is_dark_sector: false,
        is_special: false,
        tileset: None,
        tables: Vec::new(),
    }
}

#[cfg(test)]
fn mission_type(name: &str) -> MissionType {
    MissionType {
        name: name.to_owned(),
        is_endless: false,
    }
}

/// Jupiter as on the wiki (Io, Sinai, Cameria, a junction), Earth with a node without levels,
/// and a bounty.
#[cfg(test)]
fn database() -> Connection {
    let dataset = DropDataset {
        mission_types: [
            "Defense",
            "Survival",
            "Capture",
            "Solar Rail Junction",
            "Relay",
        ]
        .into_iter()
        .map(mission_type)
        .collect(),
        nodes: vec![
            node("Io", "Jupiter", "Defense", "Corpus", Some((15, 20))),
            Node {
                is_dark_sector: true,
                ..node("Sinai", "Jupiter", "Defense", "Infested", Some((20, 30)))
            },
            Node {
                is_dark_sector: true,
                ..node("Cameria", "Jupiter", "Survival", "Infested", Some((20, 30)))
            },
            node("Ananke", "Jupiter", "Capture", "Corpus", Some((25, 30))),
            node(
                "Saturn Junction",
                "Jupiter",
                "Solar Rail Junction",
                "Tenno",
                Some((1, 3)),
            ),
            node("Strata", "Earth", "Relay", "Tenno", None),
            node("Mantle", "Earth", "Capture", "Grineer or Corpus", None),
            Node {
                is_special: true,
                ..node("special:Bounty", "Cetus Bounty", "Defense", "Grineer", None)
            },
        ],
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
        &|| 1_763_500_000,
    )
    .expect("import");
    conn
}

#[cfg(test)]
fn names(conn: &Connection, filter: &StarChartFilter) -> Vec<String> {
    db::star_chart_nodes(conn, filter)
        .expect("query")
        .into_iter()
        .map(|node| node.name)
        .collect()
}

#[cfg(test)]
fn id(conn: &Connection, table: &str, name: &str) -> i64 {
    conn.query_row(
        &format!("SELECT id FROM {table} WHERE name = ?1"),
        [name],
        |row| row.get(0),
    )
    .expect("id")
}

#[test]
fn lists_playable_nodes_by_planet_then_name() {
    let conn = database();
    assert_eq!(
        names(&conn, &StarChartFilter::default()),
        ["Mantle", "Ananke", "Cameria", "Io", "Sinai"]
    );
    let io = db::star_chart_nodes(&conn, &StarChartFilter::default())
        .expect("query")
        .into_iter()
        .find(|node| node.name == "Io")
        .expect("Io");
    assert_eq!(io.planet, "Jupiter");
    assert_eq!(io.mission_type, "Defense");
    assert_eq!(io.faction.as_deref(), Some("Corpus"));
    assert_eq!((io.min_level, io.max_level), (Some(15), Some(20)));
    assert!(!io.is_dark_sector);
}

#[test]
fn level_range_keeps_overlapping_nodes() {
    let conn = database();
    let range = |min: Option<i64>, max: Option<i64>| {
        names(
            &conn,
            &StarChartFilter {
                min_level: min,
                max_level: max,
                ..StarChartFilter::default()
            },
        )
    };
    assert_eq!(
        range(Some(15), Some(25)),
        ["Ananke", "Cameria", "Io", "Sinai"]
    );
    assert_eq!(range(Some(21), Some(25)), ["Ananke", "Cameria", "Sinai"]);
    assert_eq!(range(Some(15), Some(20)), ["Cameria", "Io", "Sinai"]);
    // One bound only; nodes without levels drop out as soon as a bound is set.
    assert_eq!(range(None, Some(19)), ["Io"]);
    assert_eq!(range(Some(26), None), ["Ananke", "Cameria", "Sinai"]);
}

#[test]
fn mission_type_planet_and_faction_filters() {
    let conn = database();
    let defense = StarChartFilter {
        mission_type_id: Some(id(&conn, "mission_type", "Defense")),
        ..StarChartFilter::default()
    };
    assert_eq!(names(&conn, &defense), ["Io", "Sinai"]);
    let earth = StarChartFilter {
        planet_id: Some(id(&conn, "planet", "Earth")),
        ..StarChartFilter::default()
    };
    assert_eq!(names(&conn, &earth), ["Mantle"]);
    let corpus = StarChartFilter {
        faction: Some("corpus".to_owned()),
        ..StarChartFilter::default()
    };
    assert_eq!(names(&conn, &corpus), ["Mantle", "Ananke", "Io"]);
}

#[test]
fn dark_sector_filter() {
    let conn = database();
    let with = |dark_sectors| {
        names(
            &conn,
            &StarChartFilter {
                dark_sectors,
                ..StarChartFilter::default()
            },
        )
    };
    assert_eq!(with(NodeFilter::Only), ["Cameria", "Sinai"]);
    assert_eq!(with(NodeFilter::Exclude), ["Mantle", "Ananke", "Io"]);
}

#[test]
fn choices_only_cover_playable_nodes() {
    let conn = database();
    let choices = db::star_chart_choices(&conn).expect("choices");
    let types: Vec<&str> = choices
        .mission_types
        .iter()
        .map(|row| row.name.as_str())
        .collect();
    assert_eq!(types, ["Capture", "Defense", "Survival"]);
    let planets: Vec<&str> = choices
        .planets
        .iter()
        .map(|row| row.name.as_str())
        .collect();
    assert_eq!(planets, ["Earth", "Jupiter"]);
    assert_eq!(choices.factions, ["corpus", "grineer", "infested"]);
}
