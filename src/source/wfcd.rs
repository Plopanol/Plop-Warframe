//! `@wfcd/items` relic list (`Relics.json`): only the `vaulted` status is read, according to
//! `docs/04-REGLES-DROPS.md` (rule 17) and ADR-003.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::{DropSource, SourceError};
use crate::model::{DropDataset, Refinement, RelicStatus};

pub const WFCD_RELICS_URL: &str = "https://cdn.jsdelivr.net/npm/@wfcd/items/data/json/Relics.json";

/// Label used in errors and warnings.
pub const WFCD_RELICS: &str = "liste des reliques WFCD";

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
}
