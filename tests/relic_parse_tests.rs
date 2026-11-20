//! Reading of the `relicRewards` chapter of the DE drop tables (`tests/fixtures/README.md`).
//! Expected values were read on the DE page on the fixture download date.

use plop_warframe::model::{Rarity, Refinement, RelicReward};
use plop_warframe::source::{DeRelicSource, DropSource, parse_relic_rewards, parse_relic_vaulted};

const DE_PAGE: &str = include_str!("fixtures/droptables_de_relics.html");

#[cfg(test)]
fn reward<'a>(
    rewards: &'a [RelicReward],
    relic: &str,
    refinement: Refinement,
    item: &str,
) -> &'a RelicReward {
    rewards
        .iter()
        .find(|r| r.relic == relic && r.refinement == refinement && r.item == item)
        .unwrap_or_else(|| panic!("{item} missing in {relic} {refinement:?}"))
}

#[test]
fn reads_every_well_formed_reward() {
    let (rewards, warnings) = parse_relic_rewards(DE_PAGE).expect("parse");
    // 13 relic headers of 6 rewards, plus the readable row of the made-up Meso X1.
    assert_eq!(rewards.len(), 79);
    // Neo Z9 lacks its refinement, Meso X1 has a row without chance.
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(warnings[0].contains("Neo Z9 Relic"));
    assert!(warnings[1].contains("Broken Row Item"));
    // Neither the mission chapter before nor the key chapter after is read.
    assert!(!rewards.iter().any(|r| r.relic.contains("Mercury")));
}

#[test]
fn chances_follow_the_refinement() {
    let (rewards, _) = parse_relic_rewards(DE_PAGE).expect("parse");
    let axi = |refinement, item| reward(&rewards, "Axi A1 Relic", refinement, item).chance;
    assert_eq!(axi(Refinement::Intact, "Akstiletto Prime Barrel"), 0.11);
    assert_eq!(axi(Refinement::Exceptional, "Nikana Prime Blueprint"), 0.04);
    assert_eq!(axi(Refinement::Radiant, "Nikana Prime Blueprint"), 0.1);
    let gyre = reward(
        &rewards,
        "Lith Q3 Relic",
        Refinement::Intact,
        "Gyre Prime Blueprint",
    );
    assert_eq!(gyre.chance, 0.11);
    assert_eq!(gyre.rarity, Rarity::Uncommon);
    let common = reward(
        &rewards,
        "Lith Q3 Relic",
        Refinement::Intact,
        "Paris Prime String",
    );
    assert_eq!(common.chance, 0.2533);
}

#[test]
fn source_yields_only_relics() {
    let dataset = DeRelicSource::new(DE_PAGE).load().expect("load");
    assert_eq!(dataset.relic_rewards.len(), 79);
    assert!(dataset.nodes.is_empty() && dataset.tables.is_empty());
    assert_eq!(dataset.warnings.len(), 2);
}

#[test]
fn wfcd_status_by_relic() {
    let (status, warnings) =
        parse_relic_vaulted(include_str!("fixtures/wfcd_relics.json")).expect("parse");
    let vaulted = |relic: &str| {
        status
            .iter()
            .find(|s| s.relic == relic)
            .map(|s| s.vaulted)
            .unwrap_or_else(|| panic!("{relic} missing"))
    };
    // 7 real relics plus the made-up Meso X1; « Axi Relic » and « Void Relic » are skipped.
    assert_eq!(status.len(), 8);
    assert!(vaulted("Lith Q3 Relic"));
    assert!(!vaulted("Lith G14 Relic"));
    // Meso X1 is Intact false, Radiant true: counted vaulted, with a warning.
    assert!(vaulted("Meso X1 Relic"));
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("Meso X1 Relic"));
}
