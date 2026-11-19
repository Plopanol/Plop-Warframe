//! Spot Search queries on a small hand-made dataset, then one check on the wiki fixtures.

use plop_warframe::db::{self, ImportRun, SpotFilter};
use plop_warframe::model::{
    DropDataset, Enemy, EnemyDrop, MissionType, Node, REGION_RESOURCE, Rarity, RegionResources,
};
use plop_warframe::source::{DropSource, ENEMY_FACTIONS, WikiSource};
use rusqlite::Connection;

#[cfg(test)]
fn node(key: &str, planet: &str, mission_type: &str, faction: &str, tileset: &str) -> Node {
    Node {
        key: key.to_owned(),
        name: key.to_owned(),
        planet: planet.to_owned(),
        mission_type: mission_type.to_owned(),
        faction: Some(faction.to_owned()),
        min_level: None,
        max_level: None,
        is_dark_sector: false,
        is_special: false,
        tileset: Some(tileset.to_owned()),
        tables: Vec::new(),
    }
}

#[cfg(test)]
fn enemy(
    key: &str,
    faction: Option<&str>,
    planets: &[&str],
    tilesets: &[&str],
    drops: &[(&str, f64)],
) -> Enemy {
    Enemy {
        key: key.to_owned(),
        name: key.to_owned(),
        faction: faction.map(str::to_owned),
        planets: planets.iter().map(|s| (*s).to_owned()).collect(),
        tilesets: tilesets.iter().map(|s| (*s).to_owned()).collect(),
        drops: drops
            .iter()
            .map(|(item, chance)| EnemyDrop {
                item: (*item).to_owned(),
                category: "Mod".to_owned(),
                chance: *chance,
                rarity: Rarity::from_chance(*chance),
            })
            .collect(),
    }
}

#[cfg(test)]
fn import(dataset: &DropDataset) -> Connection {
    let mut conn = db::open_in_memory().expect("open");
    db::import_dataset(
        &mut conn,
        dataset,
        ImportRun {
            source: "test",
            source_revision: None,
        },
        &|| 1_763_500_000,
    )
    .expect("import");
    conn
}

/// Earth: two Grineer forest nodes (Survival, Capture) and an Archwing node; Mars: a Corpus
/// node. Bombard is restricted to Earth forests, Lancer goes anywhere Grineer, Ghost has no
/// known location.
#[cfg(test)]
fn database() -> Connection {
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
            MissionType {
                name: "Exterminate".to_owned(),
                is_endless: false,
            },
        ],
        nodes: vec![
            node("Capt", "Earth", "Capture", "Grineer", "Grineer Forest"),
            node("Surv", "Earth", "Survival", "Grineer", "Grineer Forest"),
            node("Wing", "Earth", "Exterminate", "Grineer", "Free Space"),
            node("Ship", "Mars", "Exterminate", "Corpus", "Corpus Ship"),
        ],
        enemies: vec![
            enemy(
                "Lancer",
                Some("grineer"),
                &[],
                &[],
                &[("Mod X", 0.03), (REGION_RESOURCE, 0.06)],
            ),
            enemy(
                "Bombard",
                Some("grineer"),
                &["Earth"],
                &["Grineer Forest"],
                &[("Mod X", 0.05)],
            ),
            enemy("Crewman", Some("corpus"), &[], &[], &[("Mod X", 0.01)]),
            enemy("Ghost", None, &[], &[], &[("Mod X", 0.5)]),
        ],
        region_resources: vec![
            RegionResources {
                planet: "Earth".to_owned(),
                resources: vec!["Ferrite".to_owned(), "Neurodes".to_owned()],
            },
            RegionResources {
                planet: "Mars".to_owned(),
                resources: vec!["Gallium".to_owned()],
            },
        ],
        ..DropDataset::default()
    };
    import(&dataset)
}

#[cfg(test)]
fn id(conn: &Connection, table: &str, name: &str) -> i64 {
    conn.query_row(
        &format!("SELECT id FROM {table} WHERE name = ?1"),
        [name],
        |row| row.get(0),
    )
    .unwrap_or_else(|_| panic!("{table} {name} missing"))
}

#[cfg(test)]
fn filter(conn: &Connection, item: &str) -> SpotFilter {
    SpotFilter {
        item_ids: vec![id(conn, "item", item)],
        ..SpotFilter::default()
    }
}

#[test]
fn enemies_are_ranked_by_per_kill_chance() {
    let conn = database();
    let rows = db::enemies_for_items(&conn, &filter(&conn, "Mod X")).expect("enemies");
    let summary: Vec<(&str, f64, i64)> = rows
        .iter()
        .map(|row| (row.enemy_name.as_str(), row.chance, row.spots))
        .collect();
    // Lancer is kept off the Archwing node; Ghost has no known spot.
    assert_eq!(
        summary,
        [
            ("Ghost", 0.5, 0),
            ("Bombard", 0.05, 2),
            ("Lancer", 0.03, 2),
            ("Crewman", 0.01, 1)
        ]
    );
}

#[test]
fn planet_filter_keeps_enemies_with_a_spot_there() {
    let conn = database();
    let mut filter = filter(&conn, "Mod X");
    filter.planet_id = Some(id(&conn, "planet", "Earth"));
    let names: Vec<String> = db::enemies_for_items(&conn, &filter)
        .expect("enemies")
        .into_iter()
        .map(|row| row.enemy_name)
        .collect();
    assert_eq!(names, ["Bombard", "Lancer"]);
}

#[test]
fn spots_rank_by_best_enemy_then_count_then_endless() {
    let conn = database();
    let rows = db::spots_for_items(&conn, &filter(&conn, "Mod X")).expect("spots");
    let summary: Vec<(&str, &str, f64, i64)> = rows
        .iter()
        .map(|row| {
            (
                row.node_name.as_str(),
                row.best_enemy.as_str(),
                row.chance,
                row.enemies,
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("Surv", "Bombard", 0.05, 2),
            ("Capt", "Bombard", 0.05, 2),
            ("Ship", "Crewman", 0.01, 1)
        ]
    );
}

#[test]
fn mission_type_and_enemy_filters_restrict_spots() {
    let conn = database();
    let mut filter = filter(&conn, "Mod X");
    filter.mission_type_id = Some(id(&conn, "mission_type", "Capture"));
    let nodes: Vec<String> = db::spots_for_items(&conn, &filter)
        .expect("spots")
        .into_iter()
        .map(|row| row.node_name)
        .collect();
    assert_eq!(nodes, ["Capt"]);

    filter.mission_type_id = None;
    filter.enemy_id = Some(id(&conn, "enemy", "Lancer"));
    let rows = db::spots_for_items(&conn, &filter).expect("spots");
    assert!(
        rows.iter()
            .all(|row| row.best_enemy == "Lancer" && row.enemies == 1)
    );
    assert_eq!(rows.len(), 2);
}

#[test]
fn region_resource_is_shared_between_the_planet_resources() {
    let conn = database();
    let filter = filter(&conn, "Neurodes");
    let enemies = db::enemies_for_items(&conn, &filter).expect("enemies");
    assert_eq!(enemies.len(), 1);
    assert_eq!(enemies[0].enemy_name, "Lancer");
    // 6 % shared between Ferrite and Neurodes.
    assert_eq!(enemies[0].chance, 0.03);
    assert!(enemies[0].estimated);

    let spots = db::spots_for_items(&conn, &filter).expect("spots");
    let nodes: Vec<&str> = spots.iter().map(|row| row.node_name.as_str()).collect();
    assert_eq!(nodes, ["Surv", "Capt"]);
    assert!(spots.iter().all(|row| row.estimated && row.chance == 0.03));
}

#[test]
fn planets_list_regular_nodes_only() {
    let conn = database();
    let planets: Vec<(String, i64)> = db::planets(&conn)
        .expect("planets")
        .into_iter()
        .map(|row| (row.name, row.nodes))
        .collect();
    assert_eq!(planets, [("Earth".to_owned(), 3), ("Mars".to_owned(), 1)]);
}

#[test]
fn no_item_gives_no_result() {
    let conn = database();
    let filter = SpotFilter::default();
    assert!(
        db::enemies_for_items(&conn, &filter)
            .expect("enemies")
            .is_empty()
    );
    assert!(
        db::spots_for_items(&conn, &filter)
            .expect("spots")
            .is_empty()
    );
}

#[test]
fn fixtures_place_aerial_commander_on_the_plains() {
    let modules = ENEMY_FACTIONS
        .iter()
        .map(|faction| {
            let path = format!(
                "{}/tests/fixtures/enemies/{faction}.lua",
                env!("CARGO_MANIFEST_DIR")
            );
            let text = std::fs::read_to_string(&path).expect("fixture");
            (format!("Module:Enemies/data/{faction}"), text)
        })
        .collect();
    let dataset = WikiSource::new(
        include_str!("fixtures/droptables_data.lua"),
        include_str!("fixtures/missions_data.lua"),
    )
    .with_enemy_modules(modules)
    .load()
    .expect("fixtures must parse");
    let conn = import(&dataset);

    let filter = filter(&conn, "Gladiator Rush");
    let enemies = db::enemies_for_items(&conn, &filter).expect("enemies");
    let commander = enemies
        .iter()
        .find(|row| row.enemy_name == "Aerial Commander")
        .expect("Aerial Commander");
    assert_eq!(commander.chance, 0.3794);
    assert!(commander.spots > 0);
    let spots = db::spots_for_items(
        &conn,
        &SpotFilter {
            enemy_id: Some(commander.enemy_id),
            ..filter
        },
    )
    .expect("spots");
    assert!(
        spots
            .iter()
            .all(|row| row.planet == "Earth" && row.best_enemy == "Aerial Commander")
    );
}
