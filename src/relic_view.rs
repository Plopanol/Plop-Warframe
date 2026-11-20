//! Texts of the Relics tab, independent of Slint so they can be tested (rules 16 and 17 of
//! `docs/04-REGLES-DROPS.md`).

use std::collections::BTreeSet;

use crate::db::{RelicRow, fold_text};
use crate::model::Refinement;
use crate::search_view::{MISSING, ResultTable, SortKey, TableRow, chance_cell, text_cell};

pub const RELIC_COLUMNS: &[&str] = &[
    "Pièce",
    "Relique",
    "Intacte",
    "Exceptionnelle",
    "Impeccable",
    "Éclatante",
    "En mission",
    "Vaulted",
];

// The column titles must follow the refinement order of `RelicRow::chances`.
const _: () = assert!(RELIC_COLUMNS.len() == Refinement::ALL.len() + 4);

/// Choices of the « En mission » and « Vaulted » filters, in [`FlagFilter`] order.
pub const FLAG_FILTER_CHOICES: [&str; 3] = ["Toutes", "Oui", "Non"];

/// Filter on a yes/no column; an unknown value only passes `All`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagFilter {
    All,
    Yes,
    No,
}

impl FlagFilter {
    /// Index of [`FLAG_FILTER_CHOICES`]; anything else means `All`.
    pub fn from_index(index: i32) -> Self {
        match index {
            1 => FlagFilter::Yes,
            2 => FlagFilter::No,
            _ => FlagFilter::All,
        }
    }

    fn accepts(self, value: Option<bool>) -> bool {
        match self {
            FlagFilter::All => true,
            FlagFilter::Yes => value == Some(true),
            FlagFilter::No => value == Some(false),
        }
    }
}

/// Rows passing both filters, in their order.
pub fn filter_rows(
    rows: &[RelicRow],
    in_mission: FlagFilter,
    vaulted: FlagFilter,
) -> Vec<RelicRow> {
    rows.iter()
        .filter(|row| in_mission.accepts(Some(row.in_mission)) && vaulted.accepts(row.vaulted))
        .cloned()
        .collect()
}

fn yes_no_cell(value: Option<bool>) -> (String, SortKey) {
    let text = match value {
        Some(true) => "Oui",
        Some(false) => "Non",
        None => MISSING,
    };
    (text.to_owned(), SortKey::text(text))
}

pub fn empty_relics() -> ResultTable {
    ResultTable {
        columns: RELIC_COLUMNS,
        rows: Vec::new(),
    }
}

/// One row per part and relic, in the query order (part, then relic name).
pub fn relic_table(rows: &[RelicRow]) -> ResultTable {
    ResultTable {
        columns: RELIC_COLUMNS,
        rows: rows
            .iter()
            .map(|row| {
                let mut cells = vec![text_cell(&row.part), text_cell(&row.relic)];
                cells.extend(row.chances.iter().map(|chance| chance_cell(*chance)));
                cells.push(yes_no_cell(Some(row.in_mission)));
                cells.push(yes_no_cell(row.vaulted));
                TableRow::new(cells)
            })
            .collect(),
    }
}

/// Warframes matching `text` without case or accents: names starting with it first, then
/// names containing it, each in list order; an empty text lists them all.
pub fn warframe_suggestions<'a>(warframes: &'a [String], text: &str) -> Vec<&'a str> {
    let needle = fold_text(text.trim());
    let (mut starts, contains): (Vec<&str>, Vec<&str>) = warframes
        .iter()
        .map(String::as_str)
        .filter(|name| fold_text(name).contains(&needle))
        .partition(|name| fold_text(name).starts_with(&needle));
    starts.extend(contains);
    starts
}

/// Line above the table, counted on the shown (filtered) rows.
pub fn summary(has_warframes: bool, warframe: Option<&str>, rows: &[RelicRow]) -> String {
    if !has_warframes {
        return "Aucune relique en base : lancez une mise à jour depuis l'onglet Données."
            .to_owned();
    }
    let Some(warframe) = warframe else {
        return "Choisissez une Warframe.".to_owned();
    };
    let relics: BTreeSet<&str> = rows.iter().map(|row| row.relic.as_str()).collect();
    let count = |keep: fn(&RelicRow) -> bool| {
        rows.iter()
            .filter(|row| keep(row))
            .map(|row| row.relic.as_str())
            .collect::<BTreeSet<_>>()
            .len()
    };
    format!(
        "{warframe} Prime : {} reliques, dont {} en mission et {} vaulted",
        relics.len(),
        count(|row| row.in_mission),
        count(|row| row.vaulted == Some(true))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(part: &str, relic: &str, vaulted: Option<bool>) -> RelicRow {
        // In the test data, a relic is in mission exactly when it is known not vaulted.
        RelicRow {
            part: part.to_owned(),
            relic: relic.to_owned(),
            chances: [Some(0.2533), Some(0.2333), Some(0.2), None],
            in_mission: vaulted == Some(false),
            vaulted,
        }
    }

    #[test]
    fn cells_show_each_refinement() {
        let table = relic_table(&[
            row("Gyre Prime Blueprint", "Lith Q3 Relic", Some(true)),
            row("Gyre Prime Blueprint", "Neo T10 Relic", None),
        ]);
        assert_eq!(
            table.rows[0].cells,
            [
                "Gyre Prime Blueprint",
                "Lith Q3 Relic",
                "25,33 %",
                "23,33 %",
                "20,00 %",
                "—",
                "Non",
                "Oui"
            ]
        );
        assert_eq!(table.rows[1].cells[7], "—");
    }

    #[test]
    fn sorting_by_chance_puts_the_best_first() {
        let mut low = row("A", "Lith A1 Relic", Some(false));
        low.chances[0] = Some(0.02);
        let mut table = relic_table(&[low, row("B", "Lith B1 Relic", Some(false))]);
        table.sort(2, true);
        assert_eq!(table.rows[0].cells[1], "Lith B1 Relic");
    }

    #[test]
    fn filters_keep_matching_rows() {
        let rows = [
            row("A", "Lith Q3 Relic", Some(true)),
            row("A", "Lith G14 Relic", Some(false)),
            row("A", "Neo T10 Relic", None),
        ];
        let relics = |mission, vaulted| {
            filter_rows(
                &rows,
                FlagFilter::from_index(mission),
                FlagFilter::from_index(vaulted),
            )
            .into_iter()
            .map(|row| row.relic)
            .collect::<Vec<_>>()
        };
        assert_eq!(relics(0, 0).len(), 3);
        assert_eq!(relics(0, 1), ["Lith Q3 Relic"]);
        assert_eq!(relics(0, 2), ["Lith G14 Relic"]);
        assert_eq!(relics(1, 0), ["Lith G14 Relic"]);
        assert_eq!(relics(2, 0), ["Lith Q3 Relic", "Neo T10 Relic"]);
        assert!(relics(1, 1).is_empty());
    }

    #[test]
    fn suggestions_put_prefixes_first() {
        let warframes: Vec<String> = ["Ash", "Gyre", "Nyx", "Styanax", "Wisp"]
            .map(str::to_owned)
            .to_vec();
        assert_eq!(warframe_suggestions(&warframes, "ny"), ["Nyx"]);
        assert_eq!(warframe_suggestions(&warframes, " A "), ["Ash", "Styanax"]);
        assert_eq!(warframe_suggestions(&warframes, "GY"), ["Gyre"]);
        assert_eq!(warframe_suggestions(&warframes, "").len(), 5);
        assert!(warframe_suggestions(&warframes, "Excalibur").is_empty());
    }

    #[test]
    fn summary_counts_distinct_relics() {
        let rows = [
            row("Gyre Prime Blueprint", "Lith Q3 Relic", Some(true)),
            row("Gyre Prime Systems Blueprint", "Lith Q3 Relic", Some(true)),
            row(
                "Gyre Prime Chassis Blueprint",
                "Lith G14 Relic",
                Some(false),
            ),
        ];
        assert_eq!(
            summary(true, Some("Gyre"), &rows),
            "Gyre Prime : 2 reliques, dont 1 en mission et 1 vaulted"
        );
        assert_eq!(summary(true, None, &[]), "Choisissez une Warframe.");
        assert!(summary(false, None, &[]).starts_with("Aucune relique"));
    }
}
