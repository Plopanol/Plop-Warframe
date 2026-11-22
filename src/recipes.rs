//! Recipe identities use the wiki's full English result names, shared with drop searches.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u32,
    /// A crafted component must not silently become a raw resource if its recipe is missing.
    pub crafted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub result: String,
    pub blueprint: String,
    pub quantity: u32,
    pub credits: u32,
    pub ingredients: Vec<Ingredient>,
}

impl Recipe {
    pub fn validate(&self) -> Result<(), RecipeError> {
        if self.result.trim().is_empty()
            || self.blueprint.trim().is_empty()
            || self.quantity == 0
            || self.ingredients.is_empty()
            || self
                .ingredients
                .iter()
                .any(|i| i.name.trim().is_empty() || i.quantity == 0)
        {
            return Err(RecipeError::Invalid);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RecipeError {
    #[error(
        "Renseignez un résultat, un schéma, une quantité produite positive et au moins un ingrédient de quantité positive."
    )]
    Invalid,
    #[error("Recette absente pour « {0} » : calcul incomplet.")]
    Missing(String),
    #[error("Cycle de fabrication autour de « {0} ».")]
    Cycle(String),
    #[error("Quantité trop grande ou recette trop profonde.")]
    Overflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftStep {
    pub result: String,
    pub needed: u64,
    pub runs: u64,
    pub ingredients: Vec<(String, u64)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CraftPlan {
    pub resources: BTreeMap<String, u64>,
    pub blueprints: BTreeMap<String, u64>,
    pub credits: u64,
    pub steps: Vec<CraftStep>,
}

fn visit<'a>(
    name: &'a str,
    recipes: &'a BTreeMap<String, Recipe>,
    path: &mut BTreeSet<&'a str>,
    done: &mut BTreeSet<&'a str>,
    order: &mut Vec<&'a str>,
) -> Result<(), RecipeError> {
    if path.contains(name) {
        return Err(RecipeError::Cycle(name.to_owned()));
    }
    if done.contains(name) {
        return Ok(());
    }
    if path.len() >= 64 {
        return Err(RecipeError::Overflow);
    }
    let recipe = recipes
        .get(name)
        .ok_or_else(|| RecipeError::Missing(name.to_owned()))?;
    recipe.validate()?;
    path.insert(name);
    for ingredient in &recipe.ingredients {
        if recipes.contains_key(&ingredient.name) {
            visit(&ingredient.name, recipes, path, done, order)?;
        } else if ingredient.crafted {
            return Err(RecipeError::Missing(ingredient.name.clone()));
        }
    }
    path.remove(name);
    done.insert(name);
    order.push(name);
    Ok(())
}

fn add(map: &mut BTreeMap<String, u64>, name: &str, quantity: u64) -> Result<(), RecipeError> {
    let current = map.entry(name.to_owned()).or_default();
    *current = current.checked_add(quantity).ok_or(RecipeError::Overflow)?;
    Ok(())
}

/// Aggregate demands before rounding shared sub-recipes up to whole batches.
pub fn plan(
    recipes: &BTreeMap<String, Recipe>,
    result: &str,
    quantity: u32,
) -> Result<CraftPlan, RecipeError> {
    if quantity == 0 {
        return Err(RecipeError::Invalid);
    }
    let mut order = Vec::new();
    visit(
        result,
        recipes,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut order,
    )?;
    let mut demand = BTreeMap::from([(result.to_owned(), u64::from(quantity))]);
    let mut plan = CraftPlan::default();
    for name in order.into_iter().rev() {
        let recipe = &recipes[name];
        let needed = demand[name];
        let runs = needed.div_ceil(u64::from(recipe.quantity));
        plan.credits = plan
            .credits
            .checked_add(
                runs.checked_mul(u64::from(recipe.credits))
                    .ok_or(RecipeError::Overflow)?,
            )
            .ok_or(RecipeError::Overflow)?;
        add(&mut plan.blueprints, &recipe.blueprint, runs)?;
        let mut ingredients = Vec::new();
        for ingredient in &recipe.ingredients {
            let amount = runs
                .checked_mul(u64::from(ingredient.quantity))
                .ok_or(RecipeError::Overflow)?;
            ingredients.push((ingredient.name.clone(), amount));
            if recipes.contains_key(&ingredient.name) {
                add(&mut demand, &ingredient.name, amount)?;
            } else {
                add(&mut plan.resources, &ingredient.name, amount)?;
            }
        }
        plan.steps.push(CraftStep {
            result: name.to_owned(),
            needed,
            runs,
            ingredients,
        });
    }
    Ok(plan)
}
