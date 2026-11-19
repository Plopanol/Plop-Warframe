//! Texts of the Spot Search tab, independent of Slint so they can be tested: planet choices,
//! enemy and node tables (rules 12 to 14 of `docs/04-REGLES-DROPS.md`).

use crate::db::{EnemyDropRow, PlanetRow, SpotRow, fold_text};
use crate::search_view::{
    MISSING, ResultTable, SortKey, TableRow, mission_type_label, percent, text_cell,
};

/// Entry of the planet selector; `id` is `None` for "every planet".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanetChoice {
    pub id: Option<i64>,
    pub label: String,
}

pub const ALL_PLANETS: &str = "Toutes les planètes";

/// "Every planet" first, then the planets by name.
pub fn planet_choices(rows: &[PlanetRow]) -> Vec<PlanetChoice> {
    let mut choices: Vec<PlanetChoice> = rows
        .iter()
        .map(|row| PlanetChoice {
            id: Some(row.id),
            label: row.name.clone(),
        })
        .collect();
    choices.sort_by_cached_key(|choice| fold_text(&choice.label));
    choices.insert(
        0,
        PlanetChoice {
            id: None,
            label: ALL_PLANETS.to_owned(),
        },
    );
    choices
}

pub const ENEMY_COLUMNS: &[&str] = &["Ennemi", "Objet", "Taux par kill", "Spots"];

pub const NODE_COLUMNS: &[&str] = &[
    "Nœud",
    "Planète",
    "Type de mission",
    "Faction",
    "Meilleur ennemi",
    "Taux par kill",
    "Ennemis",
];

/// Per-kill chance, marked when it is an equal share of a planet's resources (rule 14).
pub fn kill_chance_label(chance: f64, estimated: bool) -> String {
    let value = percent(chance);
    if estimated {
        format!("{value} (estimé)")
    } else {
        value
    }
}

fn kill_chance_cell(chance: f64, estimated: bool) -> (String, SortKey) {
    (
        kill_chance_label(chance, estimated),
        SortKey::Number(Some(chance)),
    )
}

fn count_cell(count: i64) -> (String, SortKey) {
    // Counts stay far below 2^53, so the conversion is exact.
    let number = i32::try_from(count).map(f64::from).ok();
    (count.to_string(), SortKey::Number(number))
}

pub fn empty_enemies() -> ResultTable {
    ResultTable {
        columns: ENEMY_COLUMNS,
        rows: Vec::new(),
    }
}

pub fn empty_nodes() -> ResultTable {
    ResultTable {
        columns: NODE_COLUMNS,
        rows: Vec::new(),
    }
}

/// One row per enemy and chosen item; each row keeps its enemy id.
pub fn enemy_table(rows: &[EnemyDropRow]) -> ResultTable {
    ResultTable {
        columns: ENEMY_COLUMNS,
        rows: rows
            .iter()
            .map(|row| {
                TableRow::new(vec![
                    text_cell(&row.enemy_name),
                    text_cell(&row.item_name),
                    kill_chance_cell(row.chance, row.estimated),
                    count_cell(row.spots),
                ])
                .with_id(row.enemy_id)
            })
            .collect(),
    }
}

pub fn node_table(rows: &[SpotRow]) -> ResultTable {
    ResultTable {
        columns: NODE_COLUMNS,
        rows: rows
            .iter()
            .map(|row| {
                TableRow::new(vec![
                    text_cell(&row.node_name),
                    text_cell(&row.planet),
                    text_cell(mission_type_label(&row.mission_type)),
                    text_cell(row.faction.as_deref().unwrap_or(MISSING)),
                    text_cell(&row.best_enemy),
                    kill_chance_cell(row.chance, row.estimated),
                    count_cell(row.enemies),
                ])
            })
            .collect(),
    }
}

/// Line above the tables.
pub fn summary(selected_items: usize, enemies: usize, nodes: usize) -> String {
    if selected_items == 0 {
        "Ajoutez un ou plusieurs objets pour lancer la recherche.".to_owned()
    } else {
        format!("Ennemis : {enemies} · Nœuds : {nodes}")
    }
}

/// Title of the node table: every enemy, or the one picked in the enemy table.
pub fn nodes_title(enemy: Option<&str>) -> String {
    match enemy {
        Some(name) => format!("Nœuds où trouver {name}"),
        None => "Nœuds, tous ennemis confondus".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planet(id: i64, name: &str) -> PlanetRow {
        PlanetRow {
            id,
            name: name.to_owned(),
            nodes: 1,
        }
    }

    #[test]
    fn planet_choices_start_with_all() {
        let choices = planet_choices(&[planet(2, "Mars"), planet(1, "Earth")]);
        let labels: Vec<&str> = choices.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, [ALL_PLANETS, "Earth", "Mars"]);
        assert_eq!(choices[0].id, None);
        assert_eq!(choices[1].id, Some(1));
    }

    #[test]
    fn estimated_chances_are_marked() {
        assert_eq!(kill_chance_label(0.0175, true), "1,75 % (estimé)");
        assert_eq!(kill_chance_label(0.3794, false), "37,94 %");
    }

    #[test]
    fn enemy_rows_keep_their_id_through_sorting() {
        let row = |id: i64, name: &str, chance: f64| EnemyDropRow {
            enemy_id: id,
            enemy_name: name.to_owned(),
            item_name: "Mod X".to_owned(),
            chance,
            estimated: false,
            spots: id,
        };
        let mut table = enemy_table(&[row(1, "Lancer", 0.03), row(2, "Bombard", 0.05)]);
        assert_eq!(table.rows[0].cells, ["Lancer", "Mod X", "3,00 %", "1"]);
        table.sort(0, false);
        let ids: Vec<Option<i64>> = table.rows.iter().map(|row| row.id).collect();
        assert_eq!(ids, [Some(2), Some(1)]);
        table.sort(2, true);
        assert_eq!(table.rows[0].id, Some(2));
    }

    #[test]
    fn node_rows_translate_the_mission_type() {
        let table = node_table(&[SpotRow {
            node_key: "SolNode1".to_owned(),
            node_name: "Mot".to_owned(),
            planet: "Void".to_owned(),
            mission_type: "Survival".to_owned(),
            is_endless: true,
            faction: None,
            best_enemy: "Corrupted Lancer".to_owned(),
            chance: 0.0175,
            estimated: true,
            enemies: 3,
        }]);
        assert_eq!(
            table.rows[0].cells,
            [
                "Mot",
                "Void",
                "Survie",
                MISSING,
                "Corrupted Lancer",
                "1,75 % (estimé)",
                "3"
            ]
        );
    }

    #[test]
    fn summarises_and_titles() {
        assert_eq!(
            summary(0, 0, 0),
            "Ajoutez un ou plusieurs objets pour lancer la recherche."
        );
        assert_eq!(summary(1, 4, 12), "Ennemis : 4 · Nœuds : 12");
        assert_eq!(nodes_title(Some("Lancer")), "Nœuds où trouver Lancer");
        assert_eq!(nodes_title(None), "Nœuds, tous ennemis confondus");
    }
}
