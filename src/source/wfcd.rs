//! `@wfcd/items`: every item file is stored as is (`docs/04-REGLES-DROPS.md`, rule 19, and
//! ADR-004); `Relics.json` also gives the `vaulted` status of relics (rule 18, ADR-003).

use std::collections::{BTreeMap, HashSet};

use serde::Deserialize;
use serde_json::Value;

use super::{DropSource, SourceError};
use crate::model::{DropDataset, Refinement, RelicStatus, WfcdItem};

/// Base URL of the item files; `<category>.json` and [`WFCD_FR_PATH`] are relative to it.
pub const WFCD_ITEMS_BASE_URL: &str = "https://cdn.jsdelivr.net/npm/@wfcd/items/data/json/";

/// Category files of `@wfcd/items`, without `.json`, in download and display order.
pub const WFCD_CATEGORIES: [&str; 26] = [
    "Arcanes",
    "Arch-Gun",
    "Arch-Melee",
    "Archwing",
    "Components",
    "Enemy",
    "Fish",
    "Gear",
    "Glyphs",
    "Honoria",
    "Melee",
    "Misc",
    "Mods",
    "Node",
    "Pets",
    "Primary",
    "Quests",
    "Railjack",
    "Relics",
    "Resources",
    "Secondary",
    "Sentinels",
    "SentinelWeapons",
    "Sigils",
    "Skins",
    "Warframes",
];

/// Category whose file also gives the vaulted status.
pub const WFCD_RELICS_CATEGORY: &str = "Relics";

/// French translation, relative to [`WFCD_ITEMS_BASE_URL`], and its cache file name.
pub const WFCD_FR_PATH: &str = "i18n/fr.json";
pub const WFCD_FR_FILE: &str = "fr.json";

/// Labels used in errors and warnings.
pub const WFCD_RELICS: &str = "liste des reliques WFCD";
pub const WFCD_ITEMS: &str = "items WFCD";

/// Cache file name of a category.
pub fn wfcd_category_file(category: &str) -> String {
    format!("{category}.json")
}

/// Every file to download: (cache file name, URL), the categories then the translation.
pub fn wfcd_item_files() -> Vec<(String, String)> {
    WFCD_CATEGORIES
        .iter()
        .map(|category| {
            let file = wfcd_category_file(category);
            let url = format!("{WFCD_ITEMS_BASE_URL}{file}");
            (file, url)
        })
        .chain(std::iter::once((
            WFCD_FR_FILE.to_owned(),
            format!("{WFCD_ITEMS_BASE_URL}{WFCD_FR_PATH}"),
        )))
        .collect()
}

/// Raw JSON of the category files and of the French translation.
pub struct WfcdItemsSource {
    /// (category, JSON text), e.g. (`Warframes`, content of `Warframes.json`).
    files: Vec<(String, String)>,
    fr: Option<String>,
}

impl WfcdItemsSource {
    pub fn new(files: Vec<(String, String)>, fr: Option<String>) -> Self {
        Self { files, fr }
    }
}

impl DropSource for WfcdItemsSource {
    /// Yields only `wfcd_items` and the warnings of their reading.
    fn load(&self) -> Result<DropDataset, SourceError> {
        let structure = |message: String| SourceError::Structure {
            module: WFCD_ITEMS,
            message,
        };
        let fr = match &self.fr {
            Some(text) => match serde_json::from_str::<Value>(text) {
                Ok(Value::Object(map)) => map,
                Ok(_) => return Err(structure(format!("{WFCD_FR_FILE} : objet JSON attendu"))),
                Err(error) => return Err(structure(format!("{WFCD_FR_FILE} : {error}"))),
            },
            None => serde_json::Map::new(),
        };
        let mut seen = HashSet::new();
        let mut wfcd_items = Vec::new();
        let mut warnings = Vec::new();
        for (category, text) in &self.files {
            let file = wfcd_category_file(category);
            let entries = match serde_json::from_str::<Value>(text) {
                Ok(Value::Array(entries)) => entries,
                Ok(_) => return Err(structure(format!("{file} : tableau JSON attendu"))),
                Err(error) => return Err(structure(format!("{file} : {error}"))),
            };
            for (index, entry) in entries.into_iter().enumerate() {
                let field = |key: &str| entry.get(key).and_then(Value::as_str).map(str::to_owned);
                let (Some(unique_name), Some(name)) = (field("uniqueName"), field("name")) else {
                    warnings.push(format!(
                        "{WFCD_ITEMS}, {file} : entrée n° {} sans uniqueName ou name, ignorée",
                        index + 1
                    ));
                    continue;
                };
                if !seen.insert(unique_name.clone()) {
                    warnings.push(format!(
                        "{WFCD_ITEMS}, {file} : {unique_name} déjà lu dans un autre fichier, \
                         doublon ignoré"
                    ));
                    continue;
                }
                let translation = fr.get(&unique_name);
                let json_fr = translation
                    .map(serde_json::to_string)
                    .transpose()
                    .map_err(|error| structure(format!("{WFCD_FR_FILE} : {error}")))?;
                let name_fr = translation
                    .and_then(|value| value.get("name"))
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                let json = serde_json::to_string(&entry)
                    .map_err(|error| structure(format!("{file} : {error}")))?;
                wfcd_items.push(WfcdItem {
                    unique_name,
                    category: category.clone(),
                    name,
                    name_fr,
                    json,
                    json_fr,
                });
            }
        }
        Ok(DropDataset {
            wfcd_items,
            warnings,
            ..DropDataset::default()
        })
    }
}

/// Raw JSON of `Relics.json`.
pub struct WfcdSource {
    json: String,
}

impl WfcdSource {
    pub fn new(json: impl Into<String>) -> Self {
        Self { json: json.into() }
    }
}

impl DropSource for WfcdSource {
    /// Yields only `relic_status` and the warnings of its reading.
    fn load(&self) -> Result<DropDataset, SourceError> {
        let (relic_status, warnings) = parse_relic_vaulted(&self.json)?;
        Ok(DropDataset {
            relic_status,
            warnings,
            ..DropDataset::default()
        })
    }
}

/// Entry of `Relics.json`; every other field is ignored.
#[derive(Deserialize)]
struct Entry {
    name: String,
    vaulted: Option<bool>,
}

/// Vaulted status by relic, named as DE does (`Lith Q3 Relic`), sorted by name.
///
/// The file has one entry per relic and refinement (`Lith Q3 Intact`); generic entries without
/// a refinement (`Axi Relic`, `Void Relic`) are skipped. Refinements that disagree are
/// reported and the relic counts as vaulted, since one of them can no longer be farmed.
pub fn parse_relic_vaulted(json: &str) -> Result<(Vec<RelicStatus>, Vec<String>), SourceError> {
    let entries: Vec<Entry> =
        serde_json::from_str(json).map_err(|error| SourceError::Structure {
            module: WFCD_RELICS,
            message: error.to_string(),
        })?;
    let mut by_relic: BTreeMap<String, Vec<bool>> = BTreeMap::new();
    for entry in entries {
        let Some((base, refinement)) = entry.name.rsplit_once(' ') else {
            continue;
        };
        if Refinement::from_code(refinement).is_none() {
            continue;
        }
        if let Some(vaulted) = entry.vaulted {
            by_relic
                .entry(format!("{base} Relic"))
                .or_default()
                .push(vaulted);
        }
    }
    let mut warnings = Vec::new();
    let status = by_relic
        .into_iter()
        .map(|(relic, values)| {
            let vaulted = values.iter().any(|v| *v);
            if values.iter().any(|v| *v != vaulted) {
                warnings.push(format!(
                    "{WFCD_RELICS}, {relic} : statut vaulted différent selon le raffinement, \
                     relique comptée vaulted"
                ));
            }
            RelicStatus { relic, vaulted }
        })
        .collect();
    Ok((status, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_refinements_and_skips_generic_entries() {
        let json = r#"[
            {"name": "Lith Q3 Intact", "vaulted": true, "rewards": []},
            {"name": "Lith Q3 Radiant", "vaulted": true},
            {"name": "Lith G14 Intact", "vaulted": false},
            {"name": "Axi Relic"},
            {"name": "Neo Z1 Intact"}
        ]"#;
        let (status, warnings) = parse_relic_vaulted(json).expect("parse");
        assert_eq!(
            status,
            [
                RelicStatus {
                    relic: "Lith G14 Relic".into(),
                    vaulted: false
                },
                RelicStatus {
                    relic: "Lith Q3 Relic".into(),
                    vaulted: true
                },
            ]
        );
        assert!(warnings.is_empty());
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(matches!(
            parse_relic_vaulted("{"),
            Err(SourceError::Structure { .. })
        ));
    }

    #[test]
    fn items_keep_their_raw_json_and_french_name() {
        let files = vec![
            (
                "Warframes".to_owned(),
                r#"[{"uniqueName": "/W/Excalibur", "name": "Excalibur", "health": 100}]"#
                    .to_owned(),
            ),
            (
                "Mods".to_owned(),
                r#"[{"uniqueName": "/M/Serration", "name": "Serration"}, {"uniqueName": "/M/X"},
                    {"uniqueName": "/W/Excalibur", "name": "Doublon"}]"#
                    .to_owned(),
            ),
        ];
        let fr = r#"{"/W/Excalibur": {"name": "Excalibur FR", "description": "Épée"}}"#;
        let dataset = WfcdItemsSource::new(files, Some(fr.to_owned()))
            .load()
            .expect("load");
        assert_eq!(
            dataset.wfcd_items,
            [
                WfcdItem {
                    unique_name: "/W/Excalibur".into(),
                    category: "Warframes".into(),
                    name: "Excalibur".into(),
                    name_fr: Some("Excalibur FR".into()),
                    json: r#"{"health":100,"name":"Excalibur","uniqueName":"/W/Excalibur"}"#.into(),
                    json_fr: Some(r#"{"description":"Épée","name":"Excalibur FR"}"#.into()),
                },
                WfcdItem {
                    unique_name: "/M/Serration".into(),
                    category: "Mods".into(),
                    name: "Serration".into(),
                    name_fr: None,
                    json: r#"{"name":"Serration","uniqueName":"/M/Serration"}"#.into(),
                    json_fr: None,
                },
            ]
        );
        assert_eq!(dataset.warnings.len(), 2);
        assert!(dataset.warnings[0].contains("Mods.json : entrée n° 2"));
        assert!(dataset.warnings[1].contains("doublon ignoré"));
    }

    #[test]
    fn non_array_item_file_is_an_error() {
        let files = vec![("Mods".to_owned(), "{}".to_owned())];
        assert!(matches!(
            WfcdItemsSource::new(files, None).load(),
            Err(SourceError::Structure { .. })
        ));
        let bad_fr = WfcdItemsSource::new(Vec::new(), Some("[]".to_owned())).load();
        assert!(matches!(bad_fr, Err(SourceError::Structure { .. })));
    }

    #[test]
    fn item_files_cover_categories_and_translation() {
        let files = wfcd_item_files();
        assert_eq!(files.len(), WFCD_CATEGORIES.len() + 1);
        assert_eq!(
            files[0].1,
            "https://cdn.jsdelivr.net/npm/@wfcd/items/data/json/Arcanes.json"
        );
        assert_eq!(
            files
                .last()
                .map(|(file, url)| (file.as_str(), url.as_str())),
            Some((
                "fr.json",
                "https://cdn.jsdelivr.net/npm/@wfcd/items/data/json/i18n/fr.json"
            ))
        );
    }
}
