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
    pub tables: Vec<NodeTable>,
}

/// Everything a source yields for one import.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DropDataset {
    pub mission_types: Vec<MissionType>,
    pub nodes: Vec<Node>,
    pub tables: Vec<DropTable>,
    /// Non-fatal anomalies (missing alias, malformed entry), in French for the import log.
    pub warnings: Vec<String>,
}
