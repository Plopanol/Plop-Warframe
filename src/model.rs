//! Domain types produced by a [`crate::source::DropSource`], independent of storage and UI.

/// Pool of a drop table. `None` marks a table with a single pool (see `docs/04-REGLES-DROPS.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rotation {
    A,
    B,
    C,
    None,
}

impl Rotation {
    /// Database code: `A`, `B`, `C` or `NONE`.
    pub fn code(self) -> &'static str {
        match self {
            Rotation::A => "A",
            Rotation::B => "B",
            Rotation::C => "C",
            Rotation::None => "NONE",
        }
    }

    /// Inverse of [`Self::code`].
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "A" => Some(Rotation::A),
            "B" => Some(Rotation::B),
            "C" => Some(Rotation::C),
            "NONE" => Some(Rotation::None),
            _ => None,
        }
    }
}

/// Meaning of the A/B/C pools of a table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RotationKind {
    /// Endless-mission style rotations.
    Rotation,
    /// Resource caches 1, 2, 3.
    Cache,
    /// Bounty or event stages.
    Stage,
    /// A single pool.
    None,
}

impl RotationKind {
    pub fn code(self) -> &'static str {
        match self {
            RotationKind::Rotation => "rotation",
            RotationKind::Cache => "cache",
            RotationKind::Stage => "stage",
            RotationKind::None => "none",
        }
    }

    /// Inverse of [`Self::code`].
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "rotation" => Some(RotationKind::Rotation),
            "cache" => Some(RotationKind::Cache),
            "stage" => Some(RotationKind::Stage),
            "none" => Some(RotationKind::None),
            _ => None,
        }
    }
}

/// Rarity class derived from the drop chance (wiki "Rarity Classification").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rarity {
    VeryCommon,
    Common,
    Uncommon,
    Rare,
    UltraRare,
    Legendary,
    BeyondLegendary,
}

impl Rarity {
    /// Classifies a chance expressed as a fraction between 0 and 1.
    pub fn from_chance(chance: f64) -> Self {
        // Thresholds are compared in percent rounded to 1e-4, as the wiki does, so that
        // floating noise such as 33.329999 does not change the class.
        let percent = (chance * 100.0 * 10_000.0).round() / 10_000.0;
        if percent >= 60.0 {
            Rarity::VeryCommon
        } else if percent >= 33.33 {
            Rarity::Common
        } else if percent >= 10.0 {
            Rarity::Uncommon
        } else if percent >= 2.0 {
            Rarity::Rare
        } else if percent > 1.0 {
            Rarity::UltraRare
        } else if percent > 0.1 {
            Rarity::Legendary
        } else {
            Rarity::BeyondLegendary
        }
    }

    /// Database code.
    pub fn code(self) -> &'static str {
        match self {
            Rarity::VeryCommon => "very_common",
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::UltraRare => "ultra_rare",
            Rarity::Legendary => "legendary",
            Rarity::BeyondLegendary => "beyond_legendary",
        }
    }

    /// Inverse of [`Self::code`].
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "very_common" => Some(Rarity::VeryCommon),
            "common" => Some(Rarity::Common),
            "uncommon" => Some(Rarity::Uncommon),
            "rare" => Some(Rarity::Rare),
            "ultra_rare" => Some(Rarity::UltraRare),
            "legendary" => Some(Rarity::Legendary),
            "beyond_legendary" => Some(Rarity::BeyondLegendary),
            _ => None,
        }
    }

    /// French label shown in the UI.
    pub fn label(self) -> &'static str {
        match self {
            Rarity::VeryCommon => "Très commun",
            Rarity::Common => "Commun",
            Rarity::Uncommon => "Peu commun",
            Rarity::Rare => "Rare",
            Rarity::UltraRare => "Ultra rare",
            Rarity::Legendary => "Légendaire",
            Rarity::BeyondLegendary => "Au-delà de légendaire",
        }
    }
}

/// How a node uses a drop table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableRole {
    Main,
    Cache,
    Extra,
}

impl TableRole {
    pub fn code(self) -> &'static str {
        match self {
            TableRole::Main => "main",
            TableRole::Cache => "cache",
            TableRole::Extra => "extra",
        }
    }

    /// Inverse of [`Self::code`].
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "main" => Some(TableRole::Main),
            "cache" => Some(TableRole::Cache),
            "extra" => Some(TableRole::Extra),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropEntry {
    pub rotation: Rotation,
    pub item: String,
    pub category: String,
    /// Fraction between 0 and 1.
    pub chance: f64,
    pub quantity: u32,
    pub rarity: Rarity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropTable {
    pub alias: String,
    pub name: String,
    /// Wiki `Type` of the table (mission type, bounty family…).
    pub table_type: String,
    pub tier: Option<String>,
    pub kind: RotationKind,
    pub entries: Vec<DropEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionType {
    pub name: String,
    pub is_endless: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeTable {
    pub alias: String,
    pub role: TableRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Stable key: wiki `InternalName`, or `special:<alias>` for a special mission.
    pub key: String,
    pub name: String,
    pub planet: String,
    pub mission_type: String,
    pub faction: Option<String>,
    pub min_level: Option<u32>,
    pub max_level: Option<u32>,
    pub is_dark_sector: bool,
    /// Table without a fixed node (bounty, Sortie, Arbitrations…).
    pub is_special: bool,
    /// Wiki `Tileset`, matched against an enemy's `TileSets`.
    pub tileset: Option<String>,
    pub tables: Vec<NodeTable>,
}

/// Generic drop that stands for one resource of the node's planet (`RegionResources`).
pub const REGION_RESOURCE: &str = "Region Resource";

/// Per-kill drop of an enemy.
#[derive(Debug, Clone, PartialEq)]
pub struct EnemyDrop {
    pub item: String,
    pub category: String,
    /// Per-kill fraction between 0 and 1: category chance × entry chance.
    pub chance: f64,
    pub rarity: Rarity,
}

/// Enemy of `Enemies` in `Module:DropTables/data`, placed by `Module:Enemies/data`.
#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    /// Key of the drop table (`Heavy Gunner (2)` for a variant).
    pub key: String,
    pub name: String,
    /// Normalised faction code ([`faction_code`]); `None` leaves the enemy without a spot.
    pub faction: Option<String>,
    /// Restrictions listed by the wiki; empty means anywhere its faction appears.
    pub planets: Vec<String>,
    pub tilesets: Vec<String>,
    pub drops: Vec<EnemyDrop>,
}

/// Resources a « Region Resource » drop may give on a planet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionResources {
    pub planet: String,
    pub resources: Vec<String>,
}

/// Faction code shared by nodes and enemies, so that « Kuva Grineer » enemies meet
/// « Grineer » nodes (`docs/04-REGLES-DROPS.md`, rule 13). `None` for a faction that does
/// not hold nodes (Tenno, Neutral, wildlife…).
pub fn faction_code(name: &str) -> Option<&'static str> {
    let code = match name.trim().to_lowercase().as_str() {
        "grineer" | "kuva grineer" => "grineer",
        "corpus" | "corpus amalgam" => "corpus",
        "infested" | "infestation" | "infested deimos" => "infested",
        "corrupted" | "orokin" => "orokin",
        "sentient" => "sentient",
        "the murmur" | "murmur" => "murmur",
        "narmer" => "narmer",
        "scaldra" => "scaldra",
        "techrot" => "techrot",
        "anarchs" => "anarchs",
        "duviri" => "duviri",
        _ => return None,
    };
    Some(code)
}

/// Faction codes of a node's `faction` text: « Grineer / Corpus » and « Grineer or Corpus »
/// both give the two factions.
pub fn node_faction_codes(faction: &str) -> Vec<&'static str> {
    let mut codes: Vec<&'static str> = faction
        .split(" / ")
        .flat_map(|part| part.split(" or "))
        .filter_map(faction_code)
        .collect();
    codes.sort_unstable();
    codes.dedup();
    codes
}

/// Refinement of a Void relic, which changes the chances of its rewards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Refinement {
    Intact,
    Exceptional,
    Flawless,
    Radiant,
}

impl Refinement {
    pub const ALL: [Refinement; 4] = [
        Refinement::Intact,
        Refinement::Exceptional,
        Refinement::Flawless,
        Refinement::Radiant,
    ];

    /// Database code, also the English name used by the DE drop tables.
    pub fn code(self) -> &'static str {
        match self {
            Refinement::Intact => "Intact",
            Refinement::Exceptional => "Exceptional",
            Refinement::Flawless => "Flawless",
            Refinement::Radiant => "Radiant",
        }
    }

    /// Inverse of [`Self::code`].
    pub fn from_code(code: &str) -> Option<Self> {
        Refinement::ALL
            .into_iter()
            .find(|refinement| refinement.code() == code)
    }

    /// French label shown in the UI.
    pub fn label(self) -> &'static str {
        match self {
            Refinement::Intact => "Intacte",
            Refinement::Exceptional => "Exceptionnelle",
            Refinement::Flawless => "Impeccable",
            Refinement::Radiant => "Éclatante",
        }
    }
}

/// One reward of a Void relic at a given refinement (`docs/04-REGLES-DROPS.md`, rule 16).
#[derive(Debug, Clone, PartialEq)]
pub struct RelicReward {
    /// Relic name as DE writes it, e.g. `Lith Q3 Relic`.
    pub relic: String,
    pub refinement: Refinement,
    pub item: String,
    /// Fraction between 0 and 1.
    pub chance: f64,
    pub rarity: Rarity,
}

/// Whether a relic is vaulted, i.e. no longer dropped (`docs/04-REGLES-DROPS.md`, rule 18).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelicStatus {
    /// Relic name as DE writes it, e.g. `Lith Q3 Relic`.
    pub relic: String,
    pub vaulted: bool,
}

/// Era of a relic name: `Lith` for `Lith Q3 Relic`.
pub fn relic_tier(relic: &str) -> &str {
    relic.split_whitespace().next().unwrap_or_default()
}

/// Item of `@wfcd/items`, kept with its raw JSON (`docs/04-REGLES-DROPS.md`, rule 19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfcdItem {
    /// WFCD `uniqueName`, e.g. `/Lotus/Powersuits/Excalibur/Excalibur`.
    pub unique_name: String,
    /// Source file without its extension, e.g. `Warframes`.
    pub category: String,
    pub name: String,
    /// `name` of the French translation (`i18n/fr.json`), when present.
    pub name_fr: Option<String>,
    /// Entry of the category file, compact JSON.
    pub json: String,
    /// Entry of the French translation, compact JSON.
    pub json_fr: Option<String>,
}

/// Everything a source yields for one import.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DropDataset {
    pub mission_types: Vec<MissionType>,
    pub nodes: Vec<Node>,
    pub tables: Vec<DropTable>,
    pub enemies: Vec<Enemy>,
    pub region_resources: Vec<RegionResources>,
    /// Contents of the Void relics, one entry per relic, refinement and item.
    pub relic_rewards: Vec<RelicReward>,
    /// Vaulted status of the relics, when the WFCD list is available.
    pub relic_status: Vec<RelicStatus>,
    /// Every `@wfcd/items` item, in file order.
    pub wfcd_items: Vec<WfcdItem>,
    /// Non-fatal anomalies (missing alias, malformed entry), in French for the import log.
    pub warnings: Vec<String>,
}
