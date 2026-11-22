//! Recipe identities use the wiki's full English result names, shared with drop searches.

use serde::{Deserialize, Serialize};

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
