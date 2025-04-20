use crate::*;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    recipe: &'a Recipe,
    stylesheet: &'static str,
}

impl<'a> IndexTemplate<'a> {
    pub fn recipe(recipe: &'a Recipe) -> Self {
        Self {
            recipe,
            stylesheet: "/style.css",
        }
    }
}
