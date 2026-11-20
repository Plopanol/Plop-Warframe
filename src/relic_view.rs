//! Texts of the Relics tab, independent of Slint so they can be tested (rules 16 and 17 of
//! `docs/04-REGLES-DROPS.md`).

use std::collections::BTreeSet;

use crate::db::RelicRow;
use crate::model::Refinement;
use crate::search_view::{ResultTable, SortKey, TableRow, chance_cell, text_cell};

pub const RELIC_COLUMNS: &[&str] = &[
    "Pièce",
    "Relique",
    "Intacte",
    "Exceptionnelle",
    "Impeccable",
    "Éclatante",
    "En mission",
];

// The column titles must follow the refinement order of `RelicRow::chances`.
const _: () = assert!(RELIC_COLUMNS.len() == Refinement::ALL.len() + 3);

/// « En mission » cell of a relic still dropped by an imported mission table.
const IN_MISSION: &str = "Oui";
const NOT_IN_MISSION: &str = "Non";

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
                cells.push(if row.in_mission {
                    (IN_MISSION.to_owned(), SortKey::text(IN_MISSION))
                } else {
                    (NOT_IN_MISSION.to_owned(), SortKey::text(NOT_IN_MISSION))
                });
                TableRow::new(cells)
            })
            .collect(),
    }
}

/// Line above the table.
pub fn summary(has_warframes: bool, warframe: Option<&str>, rows: &[RelicRow]) -> String {
    if !has_warframes {
        return "Aucune relique en base : lancez une mise à jour depuis l'onglet Données."
            .to_owned();
    }
    let Some(warframe) = warframe else {
        return "Choisissez une Warframe.".to_owned();
    };
    let relics: BTreeSet<&str> = rows.iter().map(|row| row.relic.as_str()).collect();
    let in_mission: BTreeSet<&str> = rows
        .iter()
        .filter(|row| row.in_mission)
        .map(|row| row.relic.as_str())
        .collect();
    format!(
        "{warframe} Prime : {} reliques, dont {} en mission",
        relics.len(),
        in_mission.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(part: &str, relic: &str, in_mission: bool) -> RelicRow {
        RelicRow {
            part: part.to_owned(),
            relic: relic.to_owned(),
            chances: [Some(0.2533), Some(0.2333), Some(0.2), None],
            in_mission,
        }
    }

    #[test]
    fn cells_show_each_refinement() {
        let table = relic_table(&[row("Gyre Prime Blueprint", "Lith Q3 Relic", false)]);
        assert_eq!(
            table.rows[0].cells,
            [
                "Gyre Prime Blueprint",
                "Lith Q3 Relic",
                "25,33 %",
                "23,33 %",
                "20,00 %",
                "—",
                "Non"
            ]
        );
    }

    #[test]
    fn sorting_by_chance_puts_the_best_first() {
        let mut low = row("A", "Lith A1 Relic", true);
        low.chances[0] = Some(0.02);
        let mut table = relic_table(&[low, row("B", "Lith B1 Relic", true)]);
        table.sort(2, true);
        assert_eq!(table.rows[0].cells[1], "Lith B1 Relic");
    }

    #[test]
    fn summary_counts_distinct_relics() {
        let rows = [
            row("Gyre Prime Blueprint", "Lith Q3 Relic", false),
            row("Gyre Prime Systems Blueprint", "Lith Q3 Relic", false),
            row("Gyre Prime Chassis Blueprint", "Axi T13 Relic", true),
        ];
        assert_eq!(
            summary(true, Some("Gyre"), &rows),
            "Gyre Prime : 2 reliques, dont 1 en mission"
        );
        assert_eq!(summary(true, None, &[]), "Choisissez une Warframe.");
        assert!(summary(false, None, &[]).starts_with("Aucune relique"));
    }
}
