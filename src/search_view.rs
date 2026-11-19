//! Texts of the Search tab, independent of Slint so they can be tested: mission type
//! translation (rule 11 of `docs/04-REGLES-DROPS.md`), result cells and column sorting, shared
//! with the Spot Search tab (`crate::spot_view`).

use std::cmp::Ordering;

use crate::db::{
    CombinedRow, ItemDropRow, ItemHit, Location, MissionTypeRow, NodeFilter, fold_text,
};
use crate::model::{Rotation, RotationKind, TableRole};

/// Shown in place of a missing value.
pub const MISSING: &str = "—";

/// French names of the common mission types; other names stay in English, as in the game.
const MISSION_TYPE_LABELS: &[(&str, &str)] = &[
    ("Alchemy", "Alchimie"),
    ("Arbitrations", "Arbitrages"),
    ("Archon Hunt", "Chasse à l'Archonte"),
    ("Arena", "Arène"),
    ("Assassination", "Assassinat"),
    ("Assault", "Assaut"),
    ("Capture", "Capture"),
    ("Defection", "Défection"),
    ("Defense", "Défense"),
    ("Disruption", "Perturbation"),
    ("Excavation", "Excavation"),
    ("Exterminate", "Extermination"),
    ("Free Roam", "Exploration libre"),
    ("Hijack", "Détournement"),
    ("Infested Salvage", "Récupération infestée"),
    ("Interception", "Interception"),
    ("Invasion", "Invasion"),
    ("Mirror Defense", "Défense miroir"),
    ("Mobile Defense", "Défense mobile"),
    ("Nightmare Mode", "Mode Cauchemar"),
    ("Pursuit", "Poursuite"),
    ("Rescue", "Sauvetage"),
    ("Rush", "Course"),
    ("Sabotage", "Sabotage"),
    ("Sanctuary Onslaught", "Assaut du Sanctuaire"),
    ("Skirmish", "Escarmouche"),
    ("Sortie", "Sortie"),
    ("Spy", "Espionnage"),
    ("Survival", "Survie"),
    ("Void Armageddon", "Armageddon du Néant"),
    ("Void Cascade", "Cascade du Néant"),
    ("Void Flood", "Inondation du Néant"),
    ("Void Storm", "Tempête du Néant"),
];

/// French name of a mission type, or its English wiki name when it has no translation.
pub fn mission_type_label(name: &str) -> &str {
    MISSION_TYPE_LABELS
        .iter()
        .find(|(english, _)| *english == name)
        .map_or(name, |(_, french)| french)
}

/// Entry of the mission type selector; `id` is `None` for "every mission type".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionTypeChoice {
    pub id: Option<i64>,
    pub label: String,
}

pub const ALL_MISSION_TYPES: &str = "Tous les types de mission";

/// Selector entries: "every type" first, then the types that have at least one node, by
/// translated name.
pub fn mission_type_choices(rows: &[MissionTypeRow]) -> Vec<MissionTypeChoice> {
    let mut choices: Vec<MissionTypeChoice> = rows
        .iter()
        .filter(|row| row.nodes > 0)
        .map(|row| MissionTypeChoice {
            id: Some(row.id),
            label: mission_type_label(&row.name).to_owned(),
        })
        .collect();
    choices.sort_by_cached_key(|choice| fold_text(&choice.label));
    choices.insert(
        0,
        MissionTypeChoice {
            id: None,
            label: ALL_MISSION_TYPES.to_owned(),
        },
    );
    choices
}

/// Result display mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SearchMode {
    #[default]
    ByItem,
    Combined,
}

impl SearchMode {
    /// From the index of the mode selector: "Par objet", "Combiné".
    pub fn from_index(index: i32) -> Self {
        if index == 1 {
            SearchMode::Combined
        } else {
            SearchMode::ByItem
        }
    }
}

/// From the index of a node class selector (special missions, Dark Sectors): "Incluses",
/// "Exclues", "Uniquement".
pub fn node_filter_from_index(index: i32) -> NodeFilter {
    match index {
        1 => NodeFilter::Exclude,
        2 => NodeFilter::Only,
        _ => NodeFilter::Include,
    }
}

/// French names of the item categories of the wiki, in the plural used by the game's menus.
const CATEGORY_LABELS: &[(&str, &str)] = &[
    ("Arcane", "Arcanes"),
    ("Ayatan Sculpture", "Sculptures Ayatan"),
    ("Blueprint", "Schémas"),
    ("Cosmetic", "Cosmétiques"),
    ("Credits", "Crédits"),
    ("Decoration", "Décorations"),
    ("Fragments", "Fragments"),
    ("Item", "Objets divers"),
    ("Mod", "Mods"),
    ("Other", "Autres"),
    ("Relic", "Reliques"),
    ("Resource", "Ressources"),
    ("Scene", "Scènes"),
    ("Sigil", "Sceaux"),
];

/// French name of an item category, or its wiki name when it has no translation.
pub fn category_label(category: &str) -> &str {
    CATEGORY_LABELS
        .iter()
        .find(|(english, _)| *english == category)
        .map_or(category, |(_, french)| french)
}

/// Entry of the item type selector; `category` is `None` for "every type".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryChoice {
    pub category: Option<String>,
    pub label: String,
}

pub const ALL_CATEGORIES: &str = "Tous les types";

/// Selector entries: "every type" first, then the categories in the given order (the most
/// populated first).
pub fn category_choices(categories: &[String]) -> Vec<CategoryChoice> {
    std::iter::once(CategoryChoice {
        category: None,
        label: ALL_CATEGORIES.to_owned(),
    })
    .chain(categories.iter().map(|category| CategoryChoice {
        category: Some(category.clone()),
        label: category_label(category).to_owned(),
    }))
    .collect()
}

/// Line of the autocompletion list.
pub fn suggestion_label(hit: &ItemHit) -> String {
    format!("{} · {}", hit.name, category_label(&hit.category))
}

/// Line above the result table.
pub fn summary(selected_items: usize, results: usize) -> String {
    if selected_items == 0 {
        "Ajoutez un ou plusieurs objets pour lancer la recherche.".to_owned()
    } else {
        format!("Résultats : {results}")
    }
}

/// Meaning of the pool, from the table kind (rule 5) and the node's link to the table.
pub fn pool_label(kind: RotationKind, role: TableRole, rotation: Rotation) -> String {
    let number = match rotation {
        Rotation::A => "1",
        Rotation::B => "2",
        Rotation::C => "3",
        Rotation::None => "",
    };
    let label = match kind {
        RotationKind::Rotation => format!("Rotation {}", rotation.code()),
        RotationKind::Cache => format!("Cache {number}"),
        RotationKind::Stage => format!("Étape {number}"),
        RotationKind::None => "Unique".to_owned(),
    };
    match role {
        TableRole::Extra => format!("{label} (supplément)"),
        TableRole::Main | TableRole::Cache => label,
    }
}

/// Chance in percent with a French decimal comma, 2 to 4 decimals: `0.0769` → `7,69 %`.
pub fn percent(chance: f64) -> String {
    let fixed = format!("{:.4}", chance * 100.0);
    let trimmed = match fixed.split_once('.') {
        Some((whole, decimals)) => {
            let kept = decimals.trim_end_matches('0');
            let kept = if kept.len() < 2 { &decimals[..2] } else { kept };
            format!("{whole},{kept}")
        }
        None => fixed,
    };
    format!("{trimmed} %")
}

/// Sort value of a cell.
#[derive(Debug, Clone, PartialEq)]
pub enum SortKey {
    /// Folded text, compared without case or accents.
    Text(String),
    /// A missing number sorts below every value.
    Number(Option<f64>),
}

impl SortKey {
    pub fn text(text: &str) -> Self {
        SortKey::Text(fold_text(text))
    }

    fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (SortKey::Text(a), SortKey::Text(b)) => a.cmp(b),
            (SortKey::Number(a), SortKey::Number(b)) => match (a, b) {
                (Some(a), Some(b)) => a.total_cmp(b),
                (a, b) => a.is_some().cmp(&b.is_some()),
            },
            (SortKey::Text(_), SortKey::Number(_)) => Ordering::Greater,
            (SortKey::Number(_), SortKey::Text(_)) => Ordering::Less,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub cells: Vec<String>,
    keys: Vec<SortKey>,
    /// Database id the row stands for, kept through sorting (the enemy of a Spot row).
    pub id: Option<i64>,
}

impl TableRow {
    pub fn new(cells: Vec<(String, SortKey)>) -> Self {
        let (cells, keys) = cells.into_iter().unzip();
        Self {
            cells,
            keys,
            id: None,
        }
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }
}

/// Result table of the Search tab; rows come in the query order (best chance first).
#[derive(Debug, Clone, PartialEq)]
pub struct ResultTable {
    pub columns: &'static [&'static str],
    pub rows: Vec<TableRow>,
}

pub const BY_ITEM_COLUMNS: &[&str] = &[
    "Objet",
    "Nœud",
    "Planète",
    "Type de mission",
    "Secteur sombre",
    "Pool",
    "Chance",
    "Rareté",
    "Quantité",
    "Chance AABC",
];

pub const COMBINED_COLUMNS: &[&str] = &[
    "Nœud",
    "Planète",
    "Type de mission",
    "Secteur sombre",
    "Pool",
    "Chance cumulée",
    "Objets",
    "Chance AABC",
];

impl ResultTable {
    pub fn empty(mode: SearchMode) -> Self {
        Self {
            columns: match mode {
                SearchMode::ByItem => BY_ITEM_COLUMNS,
                SearchMode::Combined => COMBINED_COLUMNS,
            },
            rows: Vec::new(),
        }
    }

    pub fn by_item(rows: &[ItemDropRow]) -> Self {
        Self {
            columns: BY_ITEM_COLUMNS,
            rows: rows
                .iter()
                .map(|row| {
                    let mut cells = vec![text_cell(&row.item_name)];
                    cells.extend(location_cells(&row.location));
                    cells.extend([
                        chance_cell(Some(row.chance)),
                        text_cell(row.rarity.label()),
                        (
                            row.quantity.to_string(),
                            SortKey::Number(Some(f64::from(row.quantity))),
                        ),
                        chance_cell(row.aabc),
                    ]);
                    TableRow::new(cells)
                })
                .collect(),
        }
    }

    pub fn combined(rows: &[CombinedRow]) -> Self {
        Self {
            columns: COMBINED_COLUMNS,
            rows: rows
                .iter()
                .map(|row| {
                    let mut cells = location_cells(&row.location);
                    cells.extend([
                        chance_cell(Some(row.chance)),
                        (
                            row.items.to_string(),
                            SortKey::Number(Some(f64::from(row.items))),
                        ),
                        chance_cell(row.aabc),
                    ]);
                    TableRow::new(cells)
                })
                .collect(),
        }
    }

    /// Stable sort on one column; an unknown column leaves the order unchanged.
    pub fn sort(&mut self, column: usize, descending: bool) {
        if column >= self.columns.len() {
            return;
        }
        self.rows.sort_by(|a, b| {
            let order = match (a.keys.get(column), b.keys.get(column)) {
                (Some(a), Some(b)) => a.compare(b),
                _ => Ordering::Equal,
            };
            if descending { order.reverse() } else { order }
        });
    }
}

pub fn text_cell(text: &str) -> (String, SortKey) {
    (text.to_owned(), SortKey::text(text))
}

pub fn chance_cell(chance: Option<f64>) -> (String, SortKey) {
    (
        chance.map_or_else(|| MISSING.to_owned(), percent),
        SortKey::Number(chance),
    )
}

/// "Secteur sombre" cell of a Dark Sector node; other nodes leave it empty.
const DARK_SECTOR_MARK: &str = "Oui";

fn location_cells(location: &Location) -> Vec<(String, SortKey)> {
    let pool = pool_label(location.rotation_kind, location.role, location.rotation);
    // Pools sort by kind then letter, which the label text gives.
    vec![
        text_cell(&location.node_name),
        text_cell(&location.planet),
        text_cell(mission_type_label(&location.mission_type)),
        text_cell(if location.is_dark_sector {
            DARK_SECTOR_MARK
        } else {
            ""
        }),
        text_cell(&pool),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Rarity;

    fn location(node: &str, kind: RotationKind, rotation: Rotation) -> Location {
        Location {
            node_key: node.to_owned(),
            node_name: node.to_owned(),
            planet: "Ceres".to_owned(),
            mission_type: "Survival".to_owned(),
            is_special: false,
            is_dark_sector: false,
            table_alias: "T".to_owned(),
            table_name: "T".to_owned(),
            rotation_kind: kind,
            role: TableRole::Main,
            rotation,
        }
    }

    fn item_row(node: &str, item: &str, chance: f64, aabc: Option<f64>) -> ItemDropRow {
        ItemDropRow {
            location: location(node, RotationKind::Rotation, Rotation::C),
            item_id: 1,
            item_name: item.to_owned(),
            chance,
            rarity: Rarity::from_chance(chance),
            quantity: 1,
            aabc,
        }
    }

    #[test]
    fn translates_common_mission_types_and_keeps_others() {
        assert_eq!(mission_type_label("Survival"), "Survie");
        assert_eq!(mission_type_label("Mobile Defense"), "Défense mobile");
        assert_eq!(mission_type_label("Cetus Bounty"), "Cetus Bounty");
    }

    #[test]
    fn category_choices_start_with_all_and_translate() {
        let choices =
            category_choices(&["Mod".to_owned(), "Blueprint".to_owned(), "New".to_owned()]);
        let labels: Vec<&str> = choices.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, [ALL_CATEGORIES, "Mods", "Schémas", "New"]);
        assert_eq!(choices[0].category, None);
        assert_eq!(choices[2].category.as_deref(), Some("Blueprint"));
    }

    #[test]
    fn suggestion_shows_the_french_category() {
        let hit = ItemHit {
            id: 1,
            name: "Vitality".to_owned(),
            category: "Mod".to_owned(),
        };
        assert_eq!(suggestion_label(&hit), "Vitality · Mods");
    }

    #[test]
    fn choices_start_with_all_and_skip_types_without_node() {
        let row = |id, name: &str, nodes| MissionTypeRow {
            id,
            name: name.to_owned(),
            is_endless: false,
            nodes,
            special_nodes: 0,
        };
        let choices = mission_type_choices(&[
            row(1, "Survival", 3),
            row(2, "Lunaro", 0),
            row(3, "Excavation", 2),
            row(4, "Defense", 1),
        ]);
        let labels: Vec<_> = choices.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(
            labels,
            [ALL_MISSION_TYPES, "Défense", "Excavation", "Survie"]
        );
        assert_eq!(choices[0].id, None);
        assert_eq!(choices[1].id, Some(4));
    }

    #[test]
    fn maps_selector_indexes() {
        assert_eq!(SearchMode::from_index(0), SearchMode::ByItem);
        assert_eq!(SearchMode::from_index(1), SearchMode::Combined);
        assert_eq!(node_filter_from_index(0), NodeFilter::Include);
        assert_eq!(node_filter_from_index(1), NodeFilter::Exclude);
        assert_eq!(node_filter_from_index(2), NodeFilter::Only);
        assert_eq!(node_filter_from_index(-1), NodeFilter::Include);
    }

    #[test]
    fn summarises_search() {
        assert_eq!(
            summary(0, 0),
            "Ajoutez un ou plusieurs objets pour lancer la recherche."
        );
        assert_eq!(summary(2, 15), "Résultats : 15");
    }

    #[test]
    fn formats_percent_with_french_comma() {
        assert_eq!(percent(0.0769), "7,69 %");
        assert_eq!(percent(0.25), "25,00 %");
        assert_eq!(percent(0.000123), "0,0123 %");
        assert_eq!(percent(0.012345), "1,2345 %");
        assert_eq!(percent(1.0), "100,00 %");
    }

    #[test]
    fn labels_pools_by_kind() {
        assert_eq!(
            pool_label(RotationKind::Rotation, TableRole::Main, Rotation::B),
            "Rotation B"
        );
        assert_eq!(
            pool_label(RotationKind::Cache, TableRole::Cache, Rotation::C),
            "Cache 3"
        );
        assert_eq!(
            pool_label(RotationKind::Stage, TableRole::Main, Rotation::A),
            "Étape 1"
        );
        assert_eq!(
            pool_label(RotationKind::None, TableRole::Main, Rotation::None),
            "Unique"
        );
        assert_eq!(
            pool_label(RotationKind::Rotation, TableRole::Extra, Rotation::A),
            "Rotation A (supplément)"
        );
    }

    #[test]
    fn builds_by_item_cells() {
        let table = ResultTable::by_item(&[item_row("Olympus", "Lith L8", 0.0769, Some(0.05))]);
        assert_eq!(table.columns.len(), table.rows[0].cells.len());
        assert_eq!(
            table.rows[0].cells,
            [
                "Lith L8",
                "Olympus",
                "Ceres",
                "Survie",
                "",
                "Rotation C",
                "7,69 %",
                "Rare",
                "1",
                "5,00 %"
            ]
        );
    }

    #[test]
    fn builds_combined_cells_without_aabc() {
        let row = CombinedRow {
            location: Location {
                is_dark_sector: true,
                ..location("Gabii", RotationKind::None, Rotation::None)
            },
            chance: 0.5,
            items: 2,
            aabc: None,
        };
        let table = ResultTable::combined(&[row]);
        assert_eq!(table.columns.len(), table.rows[0].cells.len());
        assert_eq!(
            table.rows[0].cells,
            [
                "Gabii", "Ceres", "Survie", "Oui", "Unique", "50,00 %", "2", MISSING
            ]
        );
    }

    #[test]
    fn sorts_numbers_and_texts() {
        let mut table = ResultTable::by_item(&[
            item_row("beta", "X", 0.1, None),
            item_row("Alpha", "X", 0.3, Some(0.2)),
            item_row("Élan", "X", 0.2, Some(0.1)),
        ]);
        let nodes = |table: &ResultTable| -> Vec<String> {
            table.rows.iter().map(|row| row.cells[1].clone()).collect()
        };

        table.sort(1, false);
        assert_eq!(nodes(&table), ["Alpha", "beta", "Élan"]);
        table.sort(6, true);
        assert_eq!(nodes(&table), ["Alpha", "Élan", "beta"]);
        // A missing AABC sorts below every value.
        table.sort(9, false);
        assert_eq!(nodes(&table), ["beta", "Élan", "Alpha"]);
        table.sort(99, true);
        assert_eq!(nodes(&table), ["beta", "Élan", "Alpha"]);
    }
}
