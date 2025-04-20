use std::path::Path;

use crate::KnockKnockError;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Recipe {
    pub title: String,
    pub category: String,
    //pub ingredients: String,
    pub preparation: String,
}

pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<Recipe>, KnockKnockError> {
    let f = std::fs::File::open(recipes_path.as_ref())?;
    let recipes = serde_json::from_reader(f)?;
    Ok(recipes)
}
