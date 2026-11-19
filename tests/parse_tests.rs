//! Parser tests on the wiki fixtures (`tests/fixtures/README.md`). Expected values were read on
//! the wiki on the fixture download date.

use std::sync::OnceLock;

use plop_warframe::model::{
    DropDataset, DropEntry, DropTable, Node, Rarity, Rotation, RotationKind, TableRole,
};
use plop_warframe::source::{DropSource, SandboxedLua, WikiSource};

const DROP_TABLES: &str = include_str!("fixtures/droptables_data.lua");
const MISSIONS: &str = include_str!("fixtures/missions_data.lua");

// Helpers count as test code for the workspace lints only under cfg(test).
#[cfg(test)]
fn dataset() -> &'static DropDataset {
    static DATASET: OnceLock<DropDataset> = OnceLock::new();
    DATASET.get_or_init(|| {
        WikiSource::new(DROP_TABLES, MISSIONS)
            .load()
            .expect("fixtures must parse")
    })
}

#[cfg(test)]
fn table(alias: &str) -> &'static DropTable {
    dataset()
        .tables
        .iter()
        .find(|table| table.alias == alias)
        .unwrap_or_else(|| panic!("table {alias} missing"))
}

#[cfg(test)]
fn node(key: &str) -> &'static Node {
    dataset()
        .nodes
        .iter()
        .find(|node| node.key == key)
        .unwrap_or_else(|| panic!("node {key} missing"))
}

#[cfg(test)]
fn entry(table: &DropTable, rotation: Rotation, item: &str) -> &'static DropEntry {
    let table = self::table(&table.alias);
    table
        .entries
        .iter()
        .find(|entry| entry.rotation == rotation && entry.item == item)
        .unwrap_or_else(|| panic!("{item} missing in {} {rotation:?}", table.alias))
}

#[test]
fn counts_match_the_fixtures() {
    let data = dataset();
    // 364 tables in `Missions`, 23 of them flagged `_IgnoreEntry`.
    assert_eq!(data.tables.len(), 341);
    // 357 map nodes plus one pseudo-node per table no node uses.
    let map_nodes = data.nodes.iter().filter(|node| !node.is_special).count();
    assert_eq!(map_nodes, 357);
    assert_eq!(data.nodes.len() - map_nodes, 167);
    assert!(data.mission_types.len() >= 40);
}

#[test]
fn node_keys_are_unique() {
    let mut keys: Vec<&str> = dataset()
        .nodes
        .iter()
        .map(|node| node.key.as_str())
        .collect();
    let total = keys.len();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), total);
}

#[test]
fn tier_one_survival_matches_the_wiki() {
    let survival = table("Survival1");
    assert_eq!(survival.name, "Tier 1 Survival");
    assert_eq!(survival.table_type, "Survival");
    assert_eq!(survival.kind, RotationKind::Rotation);

    let lith = entry(survival, Rotation::C, "Lith L8");
    assert_eq!(lith.category, "Relic");
    assert!((lith.chance - 0.1106).abs() < 1e-9);
    assert_eq!(lith.rarity, Rarity::Uncommon);
    assert_eq!(lith.quantity, 1);

    // 7.6900000000000013 % in the module.
    let lith_b = entry(survival, Rotation::B, "Lith L8");
    assert!((lith_b.chance - 0.0769).abs() < 1e-12);
    assert_eq!(lith_b.rarity, Rarity::Rare);

    let endo = entry(survival, Rotation::A, "Endo");
    assert_eq!(endo.quantity, 100);
    assert!((endo.chance - 0.5).abs() < 1e-12);
    assert_eq!(endo.rarity, Rarity::Common);
}

#[test]
fn rotation_chances_sum_to_one() {
    let survival = table("Survival1");
    for rotation in [Rotation::A, Rotation::B, Rotation::C] {
        let sum: f64 = survival
            .entries
            .iter()
            .filter(|entry| entry.rotation == rotation)
            .map(|entry| entry.chance)
            .sum();
        assert!((sum - 1.0).abs() < 0.001, "{rotation:?} sums to {sum}");
    }
}

#[test]
fn survival_node_links_its_table() {
    let apollodorus = node("SolNode94");
    assert_eq!(apollodorus.name, "Apollodorus");
    assert_eq!(apollodorus.planet, "Mercury");
    assert_eq!(apollodorus.mission_type, "Survival");
    assert_eq!(apollodorus.faction.as_deref(), Some("Infested"));
    assert!(!apollodorus.is_special);
    assert!(
        apollodorus
            .tables
            .iter()
            .any(|link| link.alias == "Survival1" && link.role == TableRole::Main)
    );
}

#[test]
fn dark_sector_and_crossfire_nodes() {
    let malva = node("ClanNode1");
    assert_eq!(malva.name, "Malva");
    assert!(malva.is_dark_sector);

    let m_prime = dataset()
        .nodes
        .iter()
        .find(|node| node.name == "M Prime")
        .expect("M Prime");
    assert_eq!(m_prime.faction.as_deref(), Some("Infested / Grineer"));
}

#[test]
fn cache_tables_use_cache_kind() {
    let caches = table("JupiterCaches");
    assert_eq!(caches.kind, RotationKind::Cache);
    assert!(dataset().nodes.iter().any(|node| {
        node.tables
            .iter()
            .any(|link| link.alias == "JupiterCaches" && link.role == TableRole::Cache)
    }));
}

#[test]
fn bounties_use_stage_kind_and_are_special() {
    let bounty = table("CetusBounty1-1");
    assert_eq!(bounty.kind, RotationKind::Stage);
    let special = node("special:CetusBounty1-1");
    assert!(special.is_special);
    assert_eq!(special.mission_type, bounty.table_type);
}

#[test]
fn single_pool_tables_have_no_rotation() {
    for table in &dataset().tables {
        if table.kind == RotationKind::None {
            assert!(
                table
                    .entries
                    .iter()
                    .all(|entry| entry.rotation == Rotation::None),
                "{}",
                table.alias
            );
        }
    }
    let sortie = table("Sortie");
    assert!(
        dataset()
            .nodes
            .iter()
            .any(|node| node.key == "special:Sortie")
    );
    assert_eq!(sortie.kind, RotationKind::None);
}

#[test]
fn endless_types_come_from_mission_types() {
    let types = &dataset().mission_types;
    let survival = types
        .iter()
        .find(|t| t.name == "Survival")
        .expect("Survival");
    assert!(survival.is_endless);
    let capture = types.iter().find(|t| t.name == "Capture").expect("Capture");
    assert!(!capture.is_endless);
}

#[test]
fn missing_and_mismatched_aliases_are_reported() {
    let warnings = &dataset().warnings;
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("« Landscape » introuvable"))
    );
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("ConjunctionSurvival1Extra") && w.contains("Alias"))
    );
}

#[test]
fn chances_are_fractions() {
    for table in &dataset().tables {
        for entry in &table.entries {
            assert!(
                (0.0..=1.0).contains(&entry.chance),
                "{} {} {}",
                table.alias,
                entry.item,
                entry.chance
            );
        }
    }
}

#[test]
fn sandbox_has_no_system_access() {
    let lua = SandboxedLua::new().expect("sandbox");
    for name in [
        "io", "os", "package", "debug", "require", "dofile", "loadfile", "load",
    ] {
        let kind: String = lua
            .eval(&format!("return type({name})"))
            .expect("type() must work");
        assert_eq!(kind, "nil", "{name} must not be reachable");
    }
}

#[test]
fn broken_module_is_an_error() {
    let result = WikiSource::new("return {", MISSIONS).load();
    assert!(result.is_err());
}

#[test]
fn rarity_thresholds_follow_the_wiki() {
    let cases = [
        (1.0, Rarity::VeryCommon),
        (0.60, Rarity::VeryCommon),
        (0.5999, Rarity::Common),
        (0.3333, Rarity::Common),
        (0.3332, Rarity::Uncommon),
        (0.10, Rarity::Uncommon),
        (0.0999, Rarity::Rare),
        (0.02, Rarity::Rare),
        (0.0184, Rarity::UltraRare),
        (0.0101, Rarity::UltraRare),
        (0.01, Rarity::Legendary),
        (0.0018, Rarity::Legendary),
        (0.001, Rarity::BeyondLegendary),
    ];
    for (chance, expected) in cases {
        assert_eq!(Rarity::from_chance(chance), expected, "{chance}");
    }
}
