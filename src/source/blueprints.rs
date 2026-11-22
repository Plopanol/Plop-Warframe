//! Wiki recipe data, executed with the same restricted Lua runtime as drop modules.

use mlua::Value;
use serde::Deserialize;

use super::{DropSource, SandboxedLua, SourceError};
use crate::model::DropDataset;
use crate::recipes::{Ingredient, Recipe};

pub const BLUEPRINTS_MODULE: &str = "Module:Blueprints/data";
pub const BLUEPRINTS_CACHE: &str = "blueprints_data.lua";

pub struct BlueprintSource<'a>(pub &'a str);

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawRecipe {
    name: String,
    result: String,
    #[serde(default)]
    credits: u32,
    parts: Vec<RawPart>,
    #[serde(default)]
    product_category: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawPart {
    name: String,
    count: u32,
    #[serde(rename = "Type")]
    kind: String,
    cost: Option<RawCost>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawCost {
    #[serde(default)]
    credits: u32,
    parts: Vec<RawPart>,
}

fn collect_recipe(
    raw: RawRecipe,
    recipes: &mut Vec<Recipe>,
    depth: usize,
) -> Result<(), super::LuaError> {
    if depth > 64 {
        return Err(mlua::Error::external("recette trop profonde").into());
    }
    let mut ingredients = Vec::new();
    for part in raw.parts {
        let name = part_name(&raw.result, &part);
        let crafted = part.cost.is_some()
            || part.kind == "WarframePart"
            || (raw.product_category == "Suits" && part.kind == "Item");
        if let Some(cost) = part.cost {
            collect_recipe(
                RawRecipe {
                    name: format!("{name} Blueprint"),
                    result: name.clone(),
                    credits: cost.credits,
                    parts: cost.parts,
                    product_category: String::new(),
                },
                recipes,
                depth + 1,
            )?;
        }
        ingredients.push(Ingredient {
            name,
            quantity: part.count,
            crafted,
        });
    }
    ingredients.sort_by(|a, b| a.name.cmp(&b.name));
    let recipe = Recipe {
        result: raw.result,
        blueprint: raw.name,
        quantity: 1,
        credits: raw.credits,
        ingredients,
    };
    recipe.validate().map_err(mlua::Error::external)?;
    recipes.push(recipe);
    Ok(())
}

fn part_name(result: &str, part: &RawPart) -> String {
    match part.kind.as_str() {
        "PrimePart" => format!("{result} {}", part.name),
        "Item" if part.name.starts_with("Prime ") => {
            format!("{} {}", result.trim_end_matches(" Prime"), part.name)
        }
        "Item"
            if matches!(
                part.name.as_str(),
                "Neuroptics"
                    | "Chassis"
                    | "Systems"
                    | "Harness"
                    | "Wings"
                    | "Carapace"
                    | "Cerebrum"
                    | "Barrel"
                    | "Receiver"
                    | "Stock"
                    | "Blade"
                    | "Handle"
                    | "Hilt"
                    | "String"
                    | "Guard"
                    | "Grip"
                    | "Link"
                    | "Lower Limb"
                    | "Upper Limb"
                    | "Limbs"
                    | "Gauntlet"
                    | "Head"
                    | "Rivet"
                    | "Aegis"
                    | "Hook"
                    | "Left Gauntlet"
                    | "Right Gauntlet"
                    | "Engine"
                    | "Disc"
                    | "Subcortex"
                    | "Glove"
                    | "Heatsink"
                    | "Motor"
                    | "Core"
            ) =>
        {
            format!("{result} {}", part.name)
        }
        _ => part.name.clone(),
    }
}

impl DropSource for BlueprintSource<'_> {
    fn load(&self) -> Result<DropDataset, SourceError> {
        let parse = || {
            let lua = SandboxedLua::new()?;
            let root = lua.run_module(BLUEPRINTS_MODULE, self.0)?;
            let mut pending = vec![(String::new(), root)];
            let mut dataset = DropDataset::default();
            while let Some((_key, value)) = pending.pop() {
                if !matches!(lua.field(&value, "Parts")?, Value::Nil) {
                    let raw = lua.convert::<RawRecipe>(value)?;
                    collect_recipe(raw, &mut dataset.recipes, 0)?;
                } else {
                    for (name, child) in lua.entries(&value)? {
                        if child.is_table() {
                            pending.push((name, child));
                        }
                    }
                }
            }
            dataset.recipes.sort_by(|a, b| a.result.cmp(&b.result));
            dataset.recipes.dedup();
            let conflicts: std::collections::BTreeSet<String> = dataset
                .recipes
                .windows(2)
                .filter(|pair| pair[0].result == pair[1].result)
                .map(|pair| pair[0].result.clone())
                .collect();
            for name in &conflicts {
                dataset.warnings.push(format!("Recettes contradictoires pour « {name} » : résultat exclu du catalogue, saisie manuelle nécessaire."));
            }
            dataset.recipes.retain(|r| !conflicts.contains(&r.result));
            Ok::<_, super::LuaError>(dataset)
        };
        parse().map_err(|source| SourceError::Lua {
            module: BLUEPRINTS_MODULE,
            source,
        })
    }
}
