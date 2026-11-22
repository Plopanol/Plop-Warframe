//! Recipe result texts, built off the UI thread together with the source queries.

use crate::{
    db,
    recipes::CraftPlan,
    search_view::{mission_type_label, percent, pool_label},
};
use rusqlite::Connection;
use std::collections::BTreeMap;

pub struct RecipePresentation {
    pub summary: String,
    pub resources: Vec<Vec<String>>,
    pub blueprints: Vec<Vec<String>>,
    pub steps: Vec<String>,
}

fn need_rows(
    conn: &Connection,
    needs: &BTreeMap<String, u64>,
    level: Option<u32>,
    mission: Option<i64>,
) -> Result<Vec<Vec<String>>, db::DbError> {
    let mut rows = Vec::new();
    for (name, quantity) in needs {
        let spots = db::recipe_spots(conn, name, level, mission)?;
        let mut mission_count = 0;
        let mut enemy_count = 0;
        let mut relic_count = 0;
        let mut first = true;
        for spot in spots {
            let count = if spot.relic.is_some() {
                &mut relic_count
            } else if spot.enemy.is_some() {
                &mut enemy_count
            } else {
                &mut mission_count
            };
            if *count >= 3 {
                continue;
            }
            *count += 1;
            let chance = if let Some((_, opening)) = &spot.relic {
                format!("{} puis {}", percent(spot.chance), percent(*opening))
            } else {
                percent(spot.chance)
            };
            let how = if let Some((relic, _)) = spot.relic {
                let pool = spot
                    .location
                    .map(|l| pool_label(l.rotation_kind, l.role, l.rotation))
                    .unwrap_or_default();
                format!("{pool} ; puis ouvrir {relic} (Intacte)")
            } else if let Some(enemy) = spot.enemy {
                if spot.estimated {
                    format!("Par kill sur {enemy} (estimé)")
                } else {
                    format!("Par kill sur {enemy}")
                }
            } else if let Some(location) = spot.location {
                format!(
                    "Récompense : {}",
                    pool_label(location.rotation_kind, location.role, location.rotation)
                )
            } else {
                String::new()
            };
            let level = match (spot.min_level, spot.max_level) {
                (Some(min), Some(max)) => format!("{min}–{max}"),
                _ => "Inconnu".to_owned(),
            };
            rows.push(vec![
                if first { name.clone() } else { String::new() },
                if first {
                    quantity.to_string()
                } else {
                    String::new()
                },
                format!(
                    "{} / {} ({})",
                    spot.planet,
                    spot.node,
                    mission_type_label(&spot.mission_type)
                ),
                level,
                how,
                chance,
            ]);
            first = false;
        }
        if first {
            rows.push(vec![
                name.clone(),
                quantity.to_string(),
                "Aucun spot connu avec ces filtres".to_owned(),
                "—".into(),
                "Source non renseignée ou hors filtre".into(),
                "—".into(),
            ]);
        }
    }
    Ok(rows)
}

pub fn present(
    conn: &Connection,
    plan: &CraftPlan,
    level: Option<u32>,
    mission: Option<i64>,
) -> Result<RecipePresentation, db::DbError> {
    let level_text = level.map_or_else(
        || "sans plafond".to_owned(),
        |v| format!("plafond {v}, par chevauchement"),
    );
    let steps = plan
        .steps
        .iter()
        .rev()
        .map(|step| {
            let ingredients = step
                .ingredients
                .iter()
                .map(|(n, q)| format!("    • {n} : {q}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "{} — besoin : {}, fabrications : {}\n{ingredients}",
                step.result, step.needed, step.runs
            )
        })
        .collect();
    let title = plan
        .steps
        .first()
        .map_or_else(String::new, |s| format!("{} × {} · ", s.result, s.needed));
    Ok(RecipePresentation {
        summary: format!(
            "{title}{} ressources ou pièces distinctes · {} schémas · {} crédits de fabrication · {level_text}. Les schémas sont à obtenir séparément ; aucun stock n’est déduit.",
            plan.resources.len(),
            plan.blueprints.len(),
            plan.credits
        ),
        resources: need_rows(conn, &plan.resources, level, mission)?,
        blueprints: need_rows(conn, &plan.blueprints, level, mission)?,
        steps,
    })
}
