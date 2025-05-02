use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use crate::RecipeError;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsonRecipe {
    id: i64,
    title: String,
    category: String,
    ingredient_amount: HashSet<String>,
    preparation: String,
}

#[derive(Clone)]
pub struct Recipe {
    pub id: i64,
    pub title: String,
    pub category: String,
    pub preparation: String,
}

pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<JsonRecipe>, RecipeError> {
    let f = std::fs::File::open(recipes_path.as_ref())?;
    let recipes = serde_json::from_reader(f)?;
    Ok(recipes)
}

impl JsonRecipe {
    pub fn to_recipe(&self) -> (Recipe, impl Iterator<Item=&str>) {
        let recipe = Recipe {
            id: self.id,
            title: self.title.clone(),
            category: self.category.clone(),
            preparation: self.preparation.clone(),
        };
        let ingredients_amounts = self.ingredient_amount.iter().map(String::deref);
        (recipe, ingredients_amounts)
    }
}
