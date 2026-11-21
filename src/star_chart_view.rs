//! Texts of the Star Chart tab, independent of Slint so they can be tested: filter choices,
//! level input and node table (rule 16 of `docs/04-REGLES-DROPS.md`).

use crate::db::{StarChartNode, fold_text};
use crate::search_view::{
    DARK_SECTOR_MARK, MISSING, ResultTable, SortKey, TableRow, mission_type_label, text_cell,
};

pub const NODE_COLUMNS: &[&str] = &[
    "Planète",
    "Nœud",
    "Type de mission",
    "Faction",
    "Niveau",
    "Secteur sombre",
];

pub const ALL_FACTIONS: &str = "Toutes les factions";

/// French names of the faction codes of `crate::model::faction_code`.
const FACTION_LABELS: &[(&str, &str)] = &[
    ("anarchs", "Anarchs"),
    ("corpus", "Corpus"),
    ("duviri", "Duviri"),
    ("grineer", "Grineer"),
    ("infested", "Infestés"),
    ("murmur", "Le Murmure"),
    ("narmer", "Narmer"),
    ("orokin", "Orokin"),
    ("scaldra", "Scaldra"),
    ("sentient", "Sentients"),
    ("techrot", "Techrot"),
];

/// French name of a faction code, or the code itself when it has none.
pub fn faction_label(code: &str) -> &str {
    FACTION_LABELS
        .iter()
        .find(|(known, _)| *known == code)
        .map_or(code, |(_, label)| label)
}

/// Entry of the faction selector; `code` is `None` for "every faction".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactionChoice {
    pub code: Option<String>,
    pub label: String,
}

/// "Every faction" first, then the factions by French name.
pub fn faction_choices(codes: &[String]) -> Vec<FactionChoice> {
    let mut choices: Vec<FactionChoice> = codes
        .iter()
        .map(|code| FactionChoice {
            code: Some(code.clone()),
            label: faction_label(code).to_owned(),
        })
        .collect();
    choices.sort_by_cached_key(|choice| fold_text(&choice.label));
    choices.insert(
        0,
        FactionChoice {
            code: None,
            label: ALL_FACTIONS.to_owned(),
        },
    );
    choices
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("niveau « {0} » invalide : saisissez un nombre entier positif")]
pub struct InvalidLevel(pub String);

/// Level typed in a bound field; an empty field sets no bound.
pub fn parse_level(text: &str) -> Result<Option<i64>, InvalidLevel> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    text.parse::<u16>()
        .map(|level| Some(i64::from(level)))
        .map_err(|_| InvalidLevel(text.to_owned()))
}

/// Level range as on the wiki: `15-20`, or a single number when both ends are equal.
pub fn level_label(min: Option<i64>, max: Option<i64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) if min == max => min.to_string(),
        (Some(min), Some(max)) => format!("{min}-{max}"),
        (Some(level), None) | (None, Some(level)) => level.to_string(),
        (None, None) => MISSING.to_owned(),
    }
}

/// Sorts by lower level, then upper level; nodes without levels come first in ascending order.
fn level_cell(min: Option<i64>, max: Option<i64>) -> (String, SortKey) {
    // Game levels fit in an i32; a larger value only saturates the sort key.
    let number = |level: i64| f64::from(i32::try_from(level).unwrap_or(i32::MAX));
    let key = min
        .or(max)
        .map(|low| number(low) * 1000.0 + number(max.unwrap_or(low)));
    (level_label(min, max), SortKey::Number(key))
}

pub fn empty_nodes() -> ResultTable {
    ResultTable {
        columns: NODE_COLUMNS,
        rows: Vec::new(),
    }
}

pub fn node_table(nodes: &[StarChartNode]) -> ResultTable {
    ResultTable {
        columns: NODE_COLUMNS,
        rows: nodes
            .iter()
            .map(|node| {
                TableRow::new(vec![
                    text_cell(&node.planet),
                    text_cell(&node.name),
                    text_cell(mission_type_label(&node.mission_type)),
                    text_cell(node.faction.as_deref().unwrap_or(MISSING)),
                    level_cell(node.min_level, node.max_level),
                    text_cell(if node.is_dark_sector {
                        DARK_SECTOR_MARK
                    } else {
                        ""
                    }),
                ])
                .with_id(node.id)
            })
            .collect(),
    }
}

/// Line above the table.
pub fn summary(has_data: bool, nodes: usize) -> String {
    if !has_data {
        return "Aucun nœud en base : lancez une mise à jour depuis l'onglet Données.".to_owned();
    }
    match nodes {
        0 => "Aucun nœud ne correspond.".to_owned(),
        1 => "1 nœud".to_owned(),
        _ => format!("{nodes} nœuds"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, levels: (Option<i64>, Option<i64>), dark: bool) -> StarChartNode {
        StarChartNode {
            id: 1,
            name: name.to_owned(),
            planet: "Jupiter".to_owned(),
            mission_type: "Defense".to_owned(),
            faction: Some("Corpus".to_owned()),
            min_level: levels.0,
            max_level: levels.1,
            is_dark_sector: dark,
        }
    }

    #[test]
    fn parses_level_bounds() {
        assert_eq!(parse_level(""), Ok(None));
        assert_eq!(parse_level(" 15 "), Ok(Some(15)));
        assert_eq!(parse_level("abc"), Err(InvalidLevel("abc".to_owned())));
        assert!(parse_level("-3").is_err());
        assert_eq!(
            InvalidLevel("x".to_owned()).to_string(),
            "niveau « x » invalide : saisissez un nombre entier positif"
        );
    }

    #[test]
    fn formats_levels_like_the_wiki() {
        assert_eq!(level_label(Some(15), Some(20)), "15-20");
        assert_eq!(level_label(Some(40), Some(40)), "40");
        assert_eq!(level_label(None, None), MISSING);
    }

    #[test]
    fn builds_node_cells() {
        let table = node_table(&[node("Io", (Some(15), Some(20)), false)]);
        assert_eq!(
            table.rows[0].cells,
            ["Jupiter", "Io", "Défense", "Corpus", "15-20", ""]
        );
        let dark = node_table(&[node("Sinai", (Some(20), Some(30)), true)]);
        assert_eq!(dark.rows[0].cells[5], DARK_SECTOR_MARK);
    }

    #[test]
    fn sorts_levels_numerically() {
        let mut table = node_table(&[
            node("A", (Some(20), Some(30)), false),
            node("B", (Some(5), Some(10)), false),
            node("C", (Some(20), Some(25)), false),
            node("D", (None, None), false),
        ]);
        table.sort(4, false);
        let order: Vec<&str> = table.rows.iter().map(|row| row.cells[1].as_str()).collect();
        assert_eq!(order, ["D", "B", "C", "A"]);
    }

    #[test]
    fn faction_choices_start_with_every_faction() {
        let choices = faction_choices(&["infested".to_owned(), "corpus".to_owned()]);
        let labels: Vec<&str> = choices.iter().map(|choice| choice.label.as_str()).collect();
        assert_eq!(labels, [ALL_FACTIONS, "Corpus", "Infestés"]);
        assert_eq!(choices[2].code.as_deref(), Some("infested"));
    }

    #[test]
    fn summary_counts_nodes() {
        assert_eq!(summary(true, 0), "Aucun nœud ne correspond.");
        assert_eq!(summary(true, 1), "1 nœud");
        assert_eq!(summary(true, 12), "12 nœuds");
    }
}
