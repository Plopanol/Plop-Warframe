//! Enemy drops and locations on the wiki fixtures (`tests/fixtures/README.md`).

use std::sync::OnceLock;

use plop_warframe::db::{self, ImportRun};
use plop_warframe::model::{DropDataset, Enemy, REGION_RESOURCE, node_faction_codes};
use plop_warframe::source::{DropSource, ENEMY_FACTIONS, WikiSource};

const DROP_TABLES: &str = include_str!("fixtures/droptables_data.lua");
const MISSIONS: &str = include_str!("fixtures/missions_data.lua");
const ENEMY_MODULES: [&str; 12] = [
    include_str!("fixtures/enemies/grineer.lua"),
    include_str!("fixtures/enemies/corpus.lua"),
    include_str!("fixtures/enemies/infestation.lua"),
    include_str!("fixtures/enemies/orokin.lua"),
    include_str!("fixtures/enemies/sentient.lua"),
    include_str!("fixtures/enemies/stalker.lua"),
    include_str!("fixtures/enemies/narmer.lua"),
    include_str!("fixtures/enemies/themurmur.lua"),
    include_str!("fixtures/enemies/techrot.lua"),
    include_str!("fixtures/enemies/scaldra.lua"),
    include_str!("fixtures/enemies/anarchs.lua"),
    include_str!("fixtures/enemies/unaffiliated.lua"),
];

#[cfg(test)]
fn dataset() -> &'static DropDataset {
    static DATASET: OnceLock<DropDataset> = OnceLock::new();
    DATASET.get_or_init(|| {
        let modules = ENEMY_FACTIONS
            .iter()
            .zip(ENEMY_MODULES)
            .map(|(faction, text)| (format!("Module:Enemies/data/{faction}"), text.to_owned()))
            .collect();
        WikiSource::new(DROP_TABLES, MISSIONS)
            .with_enemy_modules(modules)
            .load()
            .expect("fixtures must parse")
    })
}

#[cfg(test)]
fn enemy(key: &str) -> &'static Enemy {
    dataset()
        .enemies
        .iter()
        .find(|enemy| enemy.key == key)
        .unwrap_or_else(|| panic!("enemy {key} missing"))
}

#[cfg(test)]
fn chance(enemy: &Enemy, item: &str) -> f64 {
    enemy
        .drops
        .iter()
        .filter(|drop| drop.item == item)
        .map(|drop| drop.chance)
        .sum()
}

#[test]
fn per_kill_chance_is_category_chance_times_entry_chance() {
    let commander = enemy("Aerial Commander");
    // ModChance 100 % × 37.94 %.
    assert_eq!(chance(commander, "Gladiator Rush"), 0.3794);
    let gunner = enemy("Corrupted Heavy Gunner");
    // ModChance 3 % × 75.88 %.
    assert_eq!(chance(gunner, "Intruder"), 0.022764);
    // ResourceChance 7 % × 100 %.
    assert_eq!(chance(gunner, REGION_RESOURCE), 0.07);
}

#[test]
fn enemies_are_placed_by_faction_planets_and_tilesets() {
    let commander = enemy("Aerial Commander");
    assert_eq!(commander.faction.as_deref(), Some("grineer"));
    assert_eq!(commander.planets, ["Earth"]);
    assert_eq!(commander.tilesets, ["Plains of Eidolon"]);

    let gunner = enemy("Heavy Gunner");
    assert_eq!(gunner.faction.as_deref(), Some("grineer"));
    assert!(gunner.planets.is_empty() && gunner.tilesets.is_empty());

    assert_eq!(
        enemy("Corrupted Heavy Gunner").faction.as_deref(),
        Some("orokin")
    );
}

#[test]
fn numbered_variants_use_the_base_enemy() {
    let Some(variant) = dataset()
        .enemies
        .iter()
        .find(|enemy| enemy.key.ends_with(" (2)") && enemy.faction.is_some())
    else {
        panic!("no numbered variant placed");
    };
    let base = variant.key.trim_end_matches(" (2)");
    let base_faction = dataset()
        .enemies
        .iter()
        .find(|enemy| enemy.key == base)
        .and_then(|enemy| enemy.faction.clone());
    if base_faction.is_some() {
        assert_eq!(variant.faction, base_faction);
    }
}

/// Regression: `Planets = {}` and stray positional values used to reject whole modules
/// (infestation, sentient, unaffiliated), leaving their enemies without a spot.
#[test]
fn every_location_module_is_read() {
    assert!(
        !dataset()
            .warnings
            .iter()
            .any(|warning| warning.contains("illisible")),
        "{:?}",
        dataset()
            .warnings
            .iter()
            .filter(|warning| warning.contains("illisible"))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        enemy("Volatile Runner").faction.as_deref(),
        Some("infested")
    );
}

#[test]
fn malformed_location_entries_are_skipped_alone() {
    let module = r#"return {
        ["Broken"] = { General = { Planets = {}, TileSets = {}, Faction = "Grineer", 1 } },
        ["Heavy Gunner"] = { General = { Faction = "Grineer", Planets = { "Earth" } } },
    }"#;
    let dataset = WikiSource::new(DROP_TABLES, MISSIONS)
        .with_enemy_modules(vec![("test".to_owned(), module.to_owned())])
        .load()
        .expect("fixtures must parse");
    let gunner = dataset
        .enemies
        .iter()
        .find(|enemy| enemy.key == "Heavy Gunner")
        .expect("Heavy Gunner");
    assert_eq!(gunner.faction.as_deref(), Some("grineer"));
    assert_eq!(gunner.planets, ["Earth"]);
}

#[test]
fn unknown_enemies_are_reported() {
    assert!(
        dataset()
            .warnings
            .iter()
            .any(|warning| warning.contains("absent de Module:Enemies/data"))
    );
}

#[test]
fn region_resources_and_tilesets_are_read() {
    let earth = dataset()
        .region_resources
        .iter()
        .find(|region| region.planet == "Earth")
        .expect("Earth");
    assert_eq!(
        earth.resources,
        ["Ferrite", "Rubedo", "Neurodes", "Detonite Ampule"]
    );
    let e_prime = dataset()
        .nodes
        .iter()
        .find(|node| node.key == "SolNode27")
        .expect("E Prime");
    assert_eq!(e_prime.tileset.as_deref(), Some("Grineer Forest"));
}

#[test]
fn node_factions_split_lists_and_alternatives() {
    assert_eq!(
        node_faction_codes("Grineer / Corpus"),
        ["corpus", "grineer"]
    );
    assert_eq!(
        node_faction_codes("Grineer or Corpus"),
        ["corpus", "grineer"]
    );
    assert_eq!(node_faction_codes("Corrupted"), ["orokin"]);
    assert!(node_faction_codes("Tenno").is_empty());
}

#[test]
fn without_location_modules_enemies_have_no_spot() {
    let dataset = WikiSource::new(DROP_TABLES, MISSIONS)
        .load()
        .expect("fixtures must parse");
    assert!(!dataset.enemies.is_empty());
    assert!(dataset.enemies.iter().all(|enemy| enemy.faction.is_none()));
    assert!(
        !dataset
            .warnings
            .iter()
            .any(|warning| warning.contains("absent de Module:Enemies/data"))
    );
}

#[test]
fn import_stores_enemies_locations_and_regions() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = db::open_in_memory()?;
    db::import_dataset(
        &mut conn,
        dataset(),
        ImportRun {
            source: "test",
            source_revision: None,
        },
        &|| 1_763_500_000,
    )?;
    let count = |sql: &str| conn.query_row(sql, [], |row| row.get::<_, i64>(0));
    assert_eq!(
        count("SELECT count(*) FROM enemy")?,
        i64::try_from(dataset().enemies.len())?
    );
    assert_eq!(
        count(
            "SELECT count(*) FROM enemy_location l JOIN enemy e ON e.id = l.enemy_id
             WHERE e.key = 'Aerial Commander'"
        )?,
        2
    );
    assert_eq!(
        count(
            "SELECT count(*) FROM region_resource r JOIN planet p ON p.id = r.planet_id
             JOIN item i ON i.id = r.item_id WHERE p.name = 'Earth' AND i.name = 'Neurodes'"
        )?,
        1
    );
    assert_eq!(
        count(
            "SELECT count(*) FROM node_faction f JOIN node n ON n.id = f.node_id
             WHERE n.name = 'Tyana Pass'"
        )?,
        2
    );
    assert_eq!(
        count(
            "SELECT count(*) FROM node_faction f JOIN node n ON n.id = f.node_id WHERE n.is_special = 1"
        )?,
        0
    );
    Ok(())
}
