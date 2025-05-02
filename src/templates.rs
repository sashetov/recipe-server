use crate::*;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    recipe: Recipe,
    stylesheet: &'static str,
    ingredients: String
}

impl IndexTemplate {
    pub fn new(recipe: Recipe, ingredients: String) -> Self {
        Self {
            recipe,
            stylesheet: "style.css",
            ingredients,
        }
    }
}
