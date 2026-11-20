//! Relics tab queries on the wiki and DE fixtures (`docs/04-REGLES-DROPS.md`, rules 16 and 17).

use rusqlite::Connection;

use plop_warframe::db::{self, ImportRun, RelicRow};
use plop_warframe::source::{DeRelicSource, DropSource, WikiSource};

#[cfg(test)]
fn clock() -> i64 {
    1_763_500_000
}

#[cfg(test)]
fn database() -> Connection {
    let mut dataset = WikiSource::new(
        include_str!("fixtures/droptables_data.lua"),
        include_str!("fixtures/missions_data.lua"),
    )
    .load()
    .expect("wiki fixtures");
    dataset.relic_rewards = DeRelicSource::new(include_str!("fixtures/droptables_de_relics.html"))
        .load()
        .expect("DE fixture")
        .relic_rewards;
    let mut conn = db::open_in_memory().expect("open");
    let run = ImportRun {
        source: "test",
        source_revision: None,
    };
    db::import_dataset(&mut conn, &dataset, run, &clock).expect("import");
    conn
}

#[test]
fn warframes_need_neuroptics_and_chassis() {
    let warframes = db::warframes(&database()).expect("warframes");
    assert!(warframes.iter().any(|name| name == "Gyre"));
    // Sentinel (Carrier Prime Systems) and Trinity (Systems only in the fixture) are left out.
    assert!(!warframes.iter().any(|name| name == "Carrier"));
    assert!(!warframes.iter().any(|name| name == "Trinity"));
    let mut sorted = warframes.clone();
    sorted.sort_by_key(|name| name.to_lowercase());
    assert_eq!(warframes, sorted);
}

#[test]
fn gyre_parts_come_in_part_order() {
    let rows = db::relics_for_warframe(&database(), "Gyre").expect("relics");
    let summary: Vec<(&str, &str, bool)> = rows
        .iter()
        .map(|row| (row.part.as_str(), row.relic.as_str(), row.in_mission))
        .collect();
    assert_eq!(
        summary,
        [
            ("Gyre Prime Blueprint", "Lith Q3 Relic", false),
            ("Gyre Prime Neuroptics Blueprint", "Lith G14 Relic", true),
            ("Gyre Prime Chassis Blueprint", "Axi T13 Relic", false),
            ("Gyre Prime Systems Blueprint", "Meso E7 Relic", false),
        ]
    );
    let blueprint: &RelicRow = &rows[0];
    assert_eq!(
        blueprint.chances,
        [Some(0.11), Some(0.13), Some(0.17), Some(0.2)]
    );
    // Only the Intact header of Lith G14 is in the fixture.
    assert_eq!(rows[1].chances[1..], [None, None, None]);
}

#[test]
fn unknown_warframe_gives_no_row() {
    assert!(
        db::relics_for_warframe(&database(), "Excalibur")
            .expect("relics")
            .is_empty()
    );
}
