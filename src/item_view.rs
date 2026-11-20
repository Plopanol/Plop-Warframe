//! Texts of the Items tab, independent of Slint so they can be tested: category choices,
//! result table, summary and the detail of an item, its raw `@wfcd/items` JSON flattened into
//! one row per scalar value (`docs/04-REGLES-DROPS.md`, rule 19).

use std::collections::HashMap;

use serde_json::Value;

use crate::db::WfcdItemHit;
use crate::search_view::{MISSING, ResultTable, TableRow, text_cell};
use crate::source::WFCD_CATEGORIES;

pub const ITEM_COLUMNS: &[&str] = &["Nom", "Nom anglais", "Catégorie"];
pub const DETAIL_COLUMNS: &[&str] = &["Champ", "Valeur", "Français"];

/// First entry of the category selector.
pub const ALL_CATEGORIES: &str = "Toutes";

/// Rows shown at most: every keystroke reruns the search and a table of the whole base
/// (about 20 000 items) is slow to build in Slint for no reading benefit.
pub const RESULT_LIMIT: usize = 500;

/// Fields shown first in the detail, in this order. `serde_json` is built without
/// `preserve_order`, so the stored JSON keys are alphabetical and the source order is lost;
/// these identify the item and are what one reads first.
const LEADING_FIELDS: [&str; 5] = ["name", "uniqueName", "category", "type", "description"];

/// Categories present in the base, in the order of [`WFCD_CATEGORIES`]; unknown ones after,
/// alphabetically.
pub fn ordered_categories(present: &[String]) -> Vec<String> {
    let mut categories = present.to_vec();
    categories.sort_by_key(|category| {
        (
            WFCD_CATEGORIES
                .iter()
                .position(|known| known == category)
                .unwrap_or(WFCD_CATEGORIES.len()),
            category.clone(),
        )
    });
    categories.dedup();
    categories
}

/// Labels of the category selector: [`ALL_CATEGORIES`] then the categories.
pub fn category_labels(categories: &[String]) -> Vec<&str> {
    std::iter::once(ALL_CATEGORIES)
        .chain(categories.iter().map(String::as_str))
        .collect()
}

/// Category of the selector entry at `index`; `None` for « Toutes » or an unknown index.
pub fn selected_category(categories: &[String], index: i32) -> Option<&str> {
    usize::try_from(index)
        .ok()
        .and_then(|i| i.checked_sub(1))
        .and_then(|i| categories.get(i))
        .map(String::as_str)
}

pub fn empty_items() -> ResultTable {
    ResultTable {
        columns: ITEM_COLUMNS,
        rows: Vec::new(),
    }
}

/// One row per item, carrying its id: French name when known, English name, category.
pub fn item_table(hits: &[WfcdItemHit]) -> ResultTable {
    ResultTable {
        columns: ITEM_COLUMNS,
        rows: hits
            .iter()
            .map(|hit| {
                TableRow::new(vec![
                    text_cell(hit.name_fr.as_deref().unwrap_or(&hit.name)),
                    text_cell(&hit.name),
                    text_cell(&hit.category),
                ])
                .with_id(hit.id)
            })
            .collect(),
    }
}

/// Line above the tables.
pub fn summary(has_data: bool, shown: usize, total: usize) -> String {
    if !has_data {
        return "Aucun item en base : lancez une mise à jour depuis l'onglet Données.".to_owned();
    }
    match total {
        0 => "Aucun item ne correspond.".to_owned(),
        1 => "1 item".to_owned(),
        _ if shown < total => format!("{shown} premiers sur {total} affichés"),
        _ => format!("{total} items"),
    }
}

/// Title above the detail table.
pub fn detail_title(name: Option<&str>) -> String {
    match name {
        Some(name) => format!("Détail : {name}"),
        None => "Détail (cliquer un item)".to_owned(),
    }
}

pub fn empty_detail() -> ResultTable {
    ResultTable {
        columns: DETAIL_COLUMNS,
        rows: Vec::new(),
    }
}

/// Detail of an item: one row per scalar value of `json` (path, value, value of the same path
/// in `json_fr` when present). Fields only in the translation are not shown.
pub fn detail_table(json: &str, json_fr: Option<&str>) -> Result<ResultTable, serde_json::Error> {
    let value: Value = serde_json::from_str(json)?;
    let french: HashMap<String, String> = match json_fr {
        Some(text) => flatten(&serde_json::from_str(text)?).into_iter().collect(),
        None => HashMap::new(),
    };
    Ok(ResultTable {
        columns: DETAIL_COLUMNS,
        rows: flatten(&value)
            .into_iter()
            .map(|(path, text)| {
                let fr = french.get(&path).map_or("", String::as_str);
                TableRow::new(vec![text_cell(&path), text_cell(&text), text_cell(fr)])
            })
            .collect(),
    })
}

/// (path, shown value) of every scalar of `value`, depth first: `abilities[0].name`,
/// `components[2].drops[1].chance`. An empty array or object is a value of its own.
pub fn flatten(value: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    match value {
        Value::Object(map) if !map.is_empty() => {
            let leading = LEADING_FIELDS
                .iter()
                .filter_map(|key| map.get_key_value(*key));
            let others = map
                .iter()
                .filter(|(key, _)| !LEADING_FIELDS.contains(&key.as_str()));
            for (key, child) in leading.chain(others) {
                walk(key.clone(), child, &mut out);
            }
        }
        other => walk(String::new(), other, &mut out),
    }
    out
}

fn walk(path: String, value: &Value, out: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, child) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                walk(child_path, child, out);
            }
        }
        Value::Array(items) if !items.is_empty() => {
            for (index, child) in items.iter().enumerate() {
                walk(format!("{path}[{index}]"), child, out);
            }
        }
        scalar => out.push((path, scalar_text(scalar))),
    }
}

fn scalar_text(value: &Value) -> String {
    match value {
        Value::Null => MISSING.to_owned(),
        Value::Bool(true) => "Oui".to_owned(),
        Value::Bool(false) => "Non".to_owned(),
        Value::Number(number) => number.to_string(),
        // Table cells hold one line.
        Value::String(text) => text.replace("\r\n", " ").replace(['\n', '\r'], " "),
        Value::Array(_) => "[]".to_owned(),
        Value::Object(_) => "{}".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(id: i64, name: &str, name_fr: Option<&str>) -> WfcdItemHit {
        WfcdItemHit {
            id,
            name: name.to_owned(),
            name_fr: name_fr.map(str::to_owned),
            category: "Mods".to_owned(),
        }
    }

    #[test]
    fn categories_follow_the_download_order() {
        let present = ["Warframes", "Zzz", "Arcanes", "Mods", "Aaa"].map(str::to_owned);
        assert_eq!(
            ordered_categories(&present),
            ["Arcanes", "Mods", "Warframes", "Aaa", "Zzz"]
        );
        let categories = ordered_categories(&present);
        assert_eq!(category_labels(&categories)[..2], ["Toutes", "Arcanes"]);
        assert_eq!(selected_category(&categories, 0), None);
        assert_eq!(selected_category(&categories, 2), Some("Mods"));
        assert_eq!(selected_category(&categories, -1), None);
        assert_eq!(selected_category(&categories, 9), None);
    }

    #[test]
    fn rows_show_the_french_name_first_and_keep_ids() {
        let mut table = item_table(&[hit(7, "Vitality", Some("Vitalité")), hit(3, "Ash", None)]);
        assert_eq!(table.rows[0].cells, ["Vitalité", "Vitality", "Mods"]);
        assert_eq!(table.rows[1].cells, ["Ash", "Ash", "Mods"]);
        table.sort(0, false);
        assert_eq!(table.rows[0].id, Some(3));
    }

    #[test]
    fn summary_tells_truncation_and_empty_base() {
        assert_eq!(summary(true, 12, 12), "12 items");
        assert_eq!(summary(true, 1, 1), "1 item");
        assert_eq!(summary(true, 0, 0), "Aucun item ne correspond.");
        assert_eq!(summary(true, 500, 20166), "500 premiers sur 20166 affichés");
        assert!(summary(false, 0, 0).starts_with("Aucun item en base"));
    }

    #[test]
    fn flatten_gives_paths_and_shown_values() {
        let value: Value = serde_json::from_str(
            r#"{"abilities":[{"name":"Shuriken"}],"build":{"time":3600},"category":"Warframes",
                "components":[{"drops":[]},{"drops":[{"chance":0.1},{"chance":null}]}],
                "description":"Line one\nline two","isPrime":false,"name":"Ash","tags":{},
                "tradable":true,"type":"Warframe","uniqueName":"/Lotus/Ash"}"#,
        )
        .unwrap_or(Value::Null);
        let flat = flatten(&value);
        let paths: Vec<&str> = flat.iter().map(|(path, _)| path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "name",
                "uniqueName",
                "category",
                "type",
                "description",
                "abilities[0].name",
                "build.time",
                "components[0].drops",
                "components[1].drops[0].chance",
                "components[1].drops[1].chance",
                "isPrime",
                "tags",
                "tradable",
            ]
        );
        let value_of = |path: &str| {
            flat.iter()
                .find(|(p, _)| p == path)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(value_of("description"), Some("Line one line two"));
        assert_eq!(value_of("components[0].drops"), Some("[]"));
        assert_eq!(value_of("components[1].drops[1].chance"), Some("—"));
        assert_eq!(value_of("tags"), Some("{}"));
        assert_eq!(value_of("isPrime"), Some("Non"));
        assert_eq!(value_of("tradable"), Some("Oui"));
        assert_eq!(value_of("build.time"), Some("3600"));
    }

    #[test]
    fn detail_adds_the_french_value_of_the_same_path() -> Result<(), serde_json::Error> {
        let table = detail_table(
            r#"{"name":"Ash","abilities":[{"name":"Shuriken","description":"Throws"}],"health":150}"#,
            Some(r#"{"name":"Ash FR","abilities":[{"name":"Shuriken FR"}],"extra":"x"}"#),
        )?;
        let cells: Vec<Vec<String>> = table.rows.iter().map(|row| row.cells.clone()).collect();
        assert_eq!(
            cells,
            [
                ["name", "Ash", "Ash FR"],
                ["abilities[0].description", "Throws", ""],
                ["abilities[0].name", "Shuriken", "Shuriken FR"],
                ["health", "150", ""],
            ]
        );
        let untranslated = detail_table(r#"{"name":"Mag"}"#, None)?;
        assert_eq!(untranslated.rows[0].cells, ["name", "Mag", ""]);
        assert!(detail_table("not json", None).is_err());
        Ok(())
    }
}
