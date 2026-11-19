//! Wiki source: `Module:DropTables/data`, `Module:Missions/data` and the faction modules of
//! `Module:Enemies/data`, mapped according to `docs/04-REGLES-DROPS.md`.

use std::collections::{BTreeMap, HashMap, HashSet};

use mlua::Value;
use serde::Deserialize;

use super::lua::SandboxedLua;
use super::{DropSource, SourceError};
use crate::model::{
    DropDataset, DropEntry, DropTable, Enemy, EnemyDrop, MissionType, Node, NodeTable, Rarity,
    RegionResources, Rotation, RotationKind, TableRole, faction_code,
};

pub const DROP_TABLES_MODULE: &str = "Module:DropTables/data";
pub const MISSIONS_MODULE: &str = "Module:Missions/data";

/// Faction sub-modules of `Module:Enemies/data`, in the order the wiki searches them.
pub const ENEMY_FACTIONS: [&str; 12] = [
    "grineer",
    "corpus",
    "infestation",
    "orokin",
    "sentient",
    "stalker",
    "narmer",
    "themurmur",
    "techrot",
    "scaldra",
    "anarchs",
    "unaffiliated",
];

/// Raw text of the modules, as downloaded with `?action=raw`.
pub struct WikiSource {
    drop_tables: String,
    missions: String,
    /// `(page, text)` of the available `Module:Enemies/data/<faction>` modules.
    enemy_modules: Vec<(String, String)>,
}

impl WikiSource {
    pub fn new(drop_tables: impl Into<String>, missions: impl Into<String>) -> Self {
        Self {
            drop_tables: drop_tables.into(),
            missions: missions.into(),
            enemy_modules: Vec::new(),
        }
    }

    /// Adds the enemy location modules; without them enemies keep their drops but get no spot.
    pub fn with_enemy_modules(mut self, modules: Vec<(String, String)>) -> Self {
        self.enemy_modules = modules;
        self
    }

    /// Enemy locations by name; a module that cannot be read is reported and skipped,
    /// because enemy drops remain useful without a spot.
    fn enemy_infos(
        &self,
        lua: &SandboxedLua,
        warnings: &mut Vec<String>,
    ) -> BTreeMap<String, RawEnemyGeneral> {
        let mut infos = BTreeMap::new();
        for (page, code) in &self.enemy_modules {
            let entries = lua
                .run_module(page, code)
                .and_then(|value| lua.convert::<BTreeMap<String, RawEnemyInfo>>(value));
            let entries = match entries {
                Ok(entries) => entries,
                Err(error) => {
                    warnings.push(format!("Module « {page} » illisible, ignoré : {error}"));
                    continue;
                }
            };
            for (key, info) in entries {
                let Some(general) = info.general else {
                    continue;
                };
                // The first module wins, as in the wiki's own lookup (Heavy Gunner is both
                // Grineer and Unaffiliated).
                if let Some(name) = general.name.clone().filter(|name| *name != key) {
                    infos.entry(name).or_insert_with(|| general.clone());
                }
                infos.entry(key).or_insert(general);
            }
        }
        infos
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

        let mut dataset = build_dataset(tables, mission_types, nodes);

        let regions: BTreeMap<String, Vec<String>> = read_optional_field(
            &lua,
            MISSIONS_MODULE,
            &mission_data,
            "RegionResources",
            &mut dataset.warnings,
        );
        dataset.region_resources = regions
            .into_iter()
            .map(|(planet, resources)| RegionResources { planet, resources })
            .collect();

        let enemy_drops: BTreeMap<String, RawEnemyDrops> = read_optional_field(
            &lua,
            DROP_TABLES_MODULE,
            &drop_data,
            "Enemies",
            &mut dataset.warnings,
        );
        let infos = self.enemy_infos(&lua, &mut dataset.warnings);
        dataset.enemies = build_enemies(enemy_drops, &infos, &mut dataset.warnings);
        Ok(dataset)
    }
}

/// Reads a field whose absence only degrades the data: reported, then treated as empty.
fn read_optional_field<T: serde::de::DeserializeOwned + Default>(
    lua: &SandboxedLua,
    module: &'static str,
    root: &Value,
    key: &str,
    warnings: &mut Vec<String>,
) -> T {
    match read_field(lua, module, root, key) {
        Ok(value) => value,
        Err(error) => {
            warnings.push(error.to_string());
            T::default()
        }
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

#[derive(Debug, Clone, Deserialize)]
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
    tileset: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawEnemyInfo {
    general: Option<RawEnemyGeneral>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawEnemyGeneral {
    name: Option<String>,
    faction: Option<String>,
    planets: Option<OneOrMany>,
    tile_sets: Option<OneOrMany>,
}

/// Drops of one enemy: each category has its own chance per kill, then a pool.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawEnemyDrops {
    name: Option<String>,
    mod_chance: Option<Field>,
    #[serde(default)]
    mods: Vec<Vec<Field>>,
    resource_chance: Option<Field>,
    #[serde(default)]
    resources: Vec<Vec<Field>>,
    blueprint_chance: Option<Field>,
    #[serde(default)]
    blueprints: Vec<Vec<Field>>,
    item_chance: Option<Field>,
    #[serde(default)]
    items: Vec<Vec<Field>>,
    relic_chance: Option<Field>,
    #[serde(default)]
    relics: Vec<Vec<Field>>,
    pigment_chance: Option<Field>,
    #[serde(default)]
    pigments: Vec<Vec<Field>>,
    sigil_chance: Option<Field>,
    #[serde(default)]
    sigils: Vec<Vec<Field>>,
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
            tileset: raw.tileset.filter(|tileset| !tileset.trim().is_empty()),
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
                tileset: None,
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
        ..DropDataset::default()
    }
}

fn build_enemies(
    raw: BTreeMap<String, RawEnemyDrops>,
    infos: &BTreeMap<String, RawEnemyGeneral>,
    warnings: &mut Vec<String>,
) -> Vec<Enemy> {
    let mut enemies = Vec::with_capacity(raw.len());
    for (key, drops) in raw {
        let name = drops.name.clone().unwrap_or_else(|| key.clone());
        let entries = enemy_drops(&key, drops, warnings);
        if entries.is_empty() {
            continue;
        }
        // Variants are numbered by the drop tables only (`Heavy Gunner (2)`).
        let info = infos
            .get(&key)
            .or_else(|| infos.get(&name))
            .or_else(|| infos.get(base_name(&key)));
        let (faction, planets, tilesets) = match info {
            Some(info) => (
                info.faction
                    .as_deref()
                    .and_then(faction_code)
                    .map(str::to_owned),
                text_list(info.planets.clone()),
                text_list(info.tile_sets.clone()),
            ),
            None => {
                if !infos.is_empty() {
                    warnings.push(format!(
                        "Ennemi « {key} » absent de Module:Enemies/data, aucun spot"
                    ));
                }
                (None, Vec::new(), Vec::new())
            }
        };
        enemies.push(Enemy {
            key,
            name,
            faction,
            planets,
            tilesets,
            drops: entries,
        });
    }
    enemies
}

fn text_list(value: Option<OneOrMany>) -> Vec<String> {
    value
        .map(OneOrMany::into_vec)
        .unwrap_or_default()
        .into_iter()
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
        .collect()
}

/// `Heavy Gunner (2)` → `Heavy Gunner`.
fn base_name(key: &str) -> &str {
    match key
        .strip_suffix(')')
        .and_then(|rest| rest.rsplit_once(" ("))
    {
        Some((base, number))
            if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()) =>
        {
            base
        }
        _ => key,
    }
}

fn enemy_drops(key: &str, raw: RawEnemyDrops, warnings: &mut Vec<String>) -> Vec<EnemyDrop> {
    let pools = [
        (raw.mod_chance, raw.mods),
        (raw.resource_chance, raw.resources),
        (raw.blueprint_chance, raw.blueprints),
        (raw.item_chance, raw.items),
        (raw.relic_chance, raw.relics),
        (raw.pigment_chance, raw.pigments),
        (raw.sigil_chance, raw.sigils),
    ];
    let mut drops = Vec::new();
    for (pool_chance, entries) in pools {
        if entries.is_empty() {
            continue;
        }
        let Some(pool_percent) = pool_chance.as_ref().and_then(number) else {
            warnings.push(format!(
                "Ennemi « {key} » : chance de catégorie absente, pool ignoré"
            ));
            continue;
        };
        for fields in entries {
            let entry = match convert_entry(Rotation::None, &fields) {
                Ok(entry) => entry,
                Err(reason) => {
                    warnings.push(format!("Ennemi « {key} » : entrée ignorée ({reason})"));
                    continue;
                }
            };
            let chance = round_chance(pool_percent / 100.0 * entry.chance);
            if !(0.0..=1.0).contains(&chance) {
                warnings.push(format!(
                    "Ennemi « {key} » : chance invalide pour « {} »",
                    entry.item
                ));
                continue;
            }
            drops.push(EnemyDrop {
                item: entry.item,
                category: entry.category,
                chance,
                rarity: Rarity::from_chance(chance),
            });
        }
    }
    drops
}

/// Six decimals absorb the wiki's floating noise (7.6900000000000013 %) without losing any
/// value it displays.
fn round_chance(fraction: f64) -> f64 {
    (fraction * 1_000_000.0).round() / 1_000_000.0
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
    let chance = round_chance(percent / 100.0);
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
