//! Wiki source: `Module:DropTables/data` and `Module:Missions/data`, mapped according to
//! `docs/04-REGLES-DROPS.md`.

use std::collections::{BTreeMap, HashMap, HashSet};

use mlua::Value;
use serde::Deserialize;

use super::lua::SandboxedLua;
use super::{DropSource, SourceError};
use crate::model::{
    DropDataset, DropEntry, DropTable, MissionType, Node, NodeTable, Rarity, Rotation,
    RotationKind, TableRole,
};

pub const DROP_TABLES_MODULE: &str = "Module:DropTables/data";
pub const MISSIONS_MODULE: &str = "Module:Missions/data";

/// Raw text of both modules, as downloaded with `?action=raw`.
pub struct WikiSource {
    drop_tables: String,
    missions: String,
}

impl WikiSource {
    pub fn new(drop_tables: impl Into<String>, missions: impl Into<String>) -> Self {
        Self {
            drop_tables: drop_tables.into(),
            missions: missions.into(),
        }
    }
}

impl DropSource for WikiSource {
    fn load(&self) -> Result<DropDataset, SourceError> {
        let lua = SandboxedLua::new().map_err(|source| SourceError::Lua {
            module: DROP_TABLES_MODULE,
            source,
        })?;

        let drop_data = run(&lua, DROP_TABLES_MODULE, &self.drop_tables)?;
        let tables: BTreeMap<String, RawTable> =
            read_field(&lua, DROP_TABLES_MODULE, &drop_data, "Missions")?;

        let mission_data = run(&lua, MISSIONS_MODULE, &self.missions)?;
        let mission_types: BTreeMap<String, RawMissionType> =
            read_field(&lua, MISSIONS_MODULE, &mission_data, "MissionTypes")?;
        let nodes: Vec<RawNode> =
            read_field(&lua, MISSIONS_MODULE, &mission_data, "MissionDetails")?;

        Ok(build_dataset(tables, mission_types, nodes))
    }
}

fn run(lua: &SandboxedLua, module: &'static str, code: &str) -> Result<Value, SourceError> {
    lua.run_module(module, code)
        .map_err(|source| SourceError::Lua { module, source })
}

fn read_field<T: serde::de::DeserializeOwned>(
    lua: &SandboxedLua,
    module: &'static str,
    root: &Value,
    key: &str,
) -> Result<T, SourceError> {
    let value = lua
        .field(root, key)
        .map_err(|source| SourceError::Lua { module, source })?;
    if value.is_nil() {
        return Err(SourceError::Structure {
            module,
            message: format!("champ « {key} » absent"),
        });
    }
    lua.convert(value).map_err(|error| SourceError::Structure {
        module,
        message: format!("champ « {key} » : {error}"),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawTable {
    alias: Option<String>,
    name: Option<String>,
    #[serde(rename = "Type")]
    table_type: Option<String>,
    tier: Option<String>,
    #[serde(default)]
    rewards: RawRewards,
    #[serde(rename = "_IgnoreEntry", default)]
    ignore_entry: bool,
}

#[derive(Debug, Default, Deserialize)]
struct RawRewards {
    #[serde(rename = "A", default)]
    a: Vec<Vec<Field>>,
    #[serde(rename = "B", default)]
    b: Vec<Vec<Field>>,
    #[serde(rename = "C", default)]
    c: Vec<Vec<Field>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Field {
    Int(i64),
    Num(f64),
    Text(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawMissionType {
    name: Option<String>,
    #[serde(default)]
    is_endless: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn into_vec(self) -> Vec<String> {
        match self {
            OneOrMany::One(value) => vec![value],
            OneOrMany::Many(values) => values,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawNode {
    name: String,
    internal_name: Option<String>,
    planet: Option<String>,
    #[serde(rename = "Type")]
    mission_type: Option<String>,
    enemy: Option<OneOrMany>,
    min_level: Option<i64>,
    max_level: Option<i64>,
    #[serde(default)]
    is_dark_sector: bool,
    drop_table_alias: Option<OneOrMany>,
    cache_drop_table_alias: Option<OneOrMany>,
    extra_drop_table_alias: Option<OneOrMany>,
}

/// Table types whose A/B/C pools are stages rather than rotations, besides any bounty.
const STAGE_TYPES: [&str; 2] = ["Plague Star", "The Circuit"];

fn build_dataset(
    raw_tables: BTreeMap<String, RawTable>,
    raw_types: BTreeMap<String, RawMissionType>,
    raw_nodes: Vec<RawNode>,
) -> DropDataset {
    let mut warnings = Vec::new();

    let mut mission_types: Vec<MissionType> = raw_types
        .into_iter()
        .map(|(key, raw)| MissionType {
            name: raw.name.unwrap_or(key),
            is_endless: raw.is_endless,
        })
        .collect();
    mission_types.sort_by(|a, b| a.name.cmp(&b.name));

    // Nodes reference a table by its key in `Missions`; the `Alias` field is sometimes a
    // copy-paste of another table's alias, so it is only reported, never used.
    let mut tables_by_alias: BTreeMap<String, RawTable> = BTreeMap::new();
    let mut ignored: HashSet<String> = HashSet::new();
    for (key, raw) in raw_tables {
        // The wiki itself leaves these tables out of its item drop lists (unused or
        // duplicated decks).
        if raw.ignore_entry {
            ignored.insert(key);
            continue;
        }
        if let Some(alias) = raw.alias.as_deref().filter(|alias| *alias != key) {
            warnings.push(format!(
                "Table « {key} » : champ Alias « {alias} » différent de sa clé, clé retenue"
            ));
        }
        tables_by_alias.insert(key, raw);
    }

    let mut nodes = Vec::with_capacity(raw_nodes.len());
    let mut used_aliases: HashSet<String> = HashSet::new();
    let mut cache_aliases: HashSet<String> = HashSet::new();
    let mut seen_keys: HashMap<String, usize> = HashMap::new();
    for raw in raw_nodes {
        let mut key = raw
            .internal_name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| raw.name.clone());
        let count = seen_keys.entry(key.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            warnings.push(format!("Nœud « {} » : clé « {key} » en double", raw.name));
            key = format!("{key}#{count}");
        }

        let mut tables = Vec::new();
        let links = [
            (raw.drop_table_alias, TableRole::Main),
            (raw.cache_drop_table_alias, TableRole::Cache),
            (raw.extra_drop_table_alias, TableRole::Extra),
        ];
        for (aliases, role) in links {
            for alias in aliases.map(OneOrMany::into_vec).unwrap_or_default() {
                if !tables_by_alias.contains_key(&alias) {
                    let reason = if ignored.contains(&alias) {
                        "marquée _IgnoreEntry"
                    } else {
                        "introuvable"
                    };
                    warnings.push(format!(
                        "Nœud « {} » : table « {alias} » {reason}, lien ignoré",
                        raw.name
                    ));
                    continue;
                }
                if role == TableRole::Cache {
                    cache_aliases.insert(alias.clone());
                }
                used_aliases.insert(alias.clone());
                let link = NodeTable { alias, role };
                if !tables.contains(&link) {
                    tables.push(link);
                }
            }
        }

        nodes.push(Node {
            key,
            name: raw.name,
            planet: raw.planet.unwrap_or_default(),
            mission_type: raw.mission_type.unwrap_or_default(),
            faction: raw.enemy.and_then(faction),
            min_level: raw.min_level.and_then(|level| u32::try_from(level).ok()),
            max_level: raw.max_level.and_then(|level| u32::try_from(level).ok()),
            is_dark_sector: raw.is_dark_sector,
            is_special: false,
            tables,
        });
    }

    let mut tables = Vec::with_capacity(tables_by_alias.len());
    for (alias, raw) in tables_by_alias {
        let name = raw.name.unwrap_or_else(|| alias.clone());
        let table_type = raw.table_type.unwrap_or_default();
        let kind = rotation_kind(cache_aliases.contains(&alias), &table_type, &raw.rewards);
        let entries = convert_entries(&alias, kind, raw.rewards, &mut warnings);

        if !used_aliases.contains(&alias) {
            nodes.push(Node {
                key: format!("special:{alias}"),
                name: name.clone(),
                planet: table_type.clone(),
                mission_type: table_type.clone(),
                faction: None,
                min_level: None,
                max_level: None,
                is_dark_sector: false,
                is_special: true,
                tables: vec![NodeTable {
                    alias: alias.clone(),
                    role: TableRole::Main,
                }],
            });
        }

        tables.push(DropTable {
            alias,
            name,
            table_type,
            tier: raw.tier,
            kind,
            entries,
        });
    }
    nodes.sort_by(|a, b| a.key.cmp(&b.key));

    DropDataset {
        mission_types,
        nodes,
        tables,
        warnings,
    }
}

fn faction(enemy: OneOrMany) -> Option<String> {
    let names: Vec<String> = enemy
        .into_vec()
        .into_iter()
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect();
    if names.is_empty() {
        None
    } else {
        Some(names.join(" / "))
    }
}

fn rotation_kind(is_cache: bool, table_type: &str, rewards: &RawRewards) -> RotationKind {
    if is_cache {
        RotationKind::Cache
    } else if table_type.contains("Bounty") || STAGE_TYPES.contains(&table_type) {
        RotationKind::Stage
    } else if rewards.b.is_empty() && rewards.c.is_empty() {
        RotationKind::None
    } else {
        RotationKind::Rotation
    }
}

fn convert_entries(
    alias: &str,
    kind: RotationKind,
    rewards: RawRewards,
    warnings: &mut Vec<String>,
) -> Vec<DropEntry> {
    let pools = [
        (Rotation::A, rewards.a),
        (Rotation::B, rewards.b),
        (Rotation::C, rewards.c),
    ];
    let mut entries = Vec::new();
    for (pool, fields) in pools {
        let rotation = if kind == RotationKind::None {
            Rotation::None
        } else {
            pool
        };
        for raw in fields {
            match convert_entry(rotation, &raw) {
                Ok(entry) => entries.push(entry),
                Err(reason) => warnings.push(format!(
                    "Table « {alias} », pool {} : entrée ignorée ({reason})",
                    pool.code()
                )),
            }
        }
    }
    entries
}

fn convert_entry(rotation: Rotation, fields: &[Field]) -> Result<DropEntry, String> {
    let (Some(Field::Text(item)), Some(Field::Text(category)), Some(chance)) =
        (fields.first(), fields.get(1), fields.get(2))
    else {
        return Err(format!(
            "{} champ(s), nom, catégorie et chance attendus",
            fields.len()
        ));
    };
    let percent = number(chance).ok_or_else(|| format!("« {item} » : chance non numérique"))?;
    if !percent.is_finite() || percent < 0.0 {
        return Err(format!("« {item} » : chance invalide {percent}"));
    }
    let quantity = match fields.get(3) {
        None => 1,
        Some(field) => quantity(field).ok_or_else(|| format!("« {item} » : quantité invalide"))?,
    };
    // Six decimals absorb the wiki's floating noise (7.6900000000000013 %) without losing
    // any value it displays.
    let chance = (percent / 100.0 * 1_000_000.0).round() / 1_000_000.0;
    Ok(DropEntry {
        rotation,
        item: item.trim().to_owned(),
        category: category.trim().to_owned(),
        chance,
        quantity,
        rarity: Rarity::from_chance(chance),
    })
}

fn number(field: &Field) -> Option<f64> {
    match field {
        Field::Int(value) => i32::try_from(*value).ok().map(f64::from),
        Field::Num(value) => Some(*value),
        Field::Text(_) => None,
    }
}

fn quantity(field: &Field) -> Option<u32> {
    match field {
        Field::Int(value) => u32::try_from(*value).ok().filter(|q| *q > 0),
        // Lua may store an integral quantity as a float (`3.0`); go through text to avoid a
        // lossy float-to-int cast.
        Field::Num(value) if value.fract() == 0.0 => {
            format!("{value:.0}").parse::<u32>().ok().filter(|q| *q > 0)
        }
        _ => None,
    }
}
