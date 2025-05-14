use crate::*;

use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use crate::RecipeError;

use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn new(recipe: Recipe, ingredients: Vec<String>) -> Self {
        let ingredients = ingredients.into_iter().collect();
        Self {
            id: recipe.id,
            title: recipe.title,
            category: recipe.category,
            ingredient_amount: ingredients,
            preparation: recipe.preparation,
        }
    }

    pub fn to_recipe(&self) -> (Recipe, impl Iterator<Item=&str>) {
        let recipe = Recipe {
            id: self.id.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            preparation: self.preparation.clone(),
        };
        let ingredient_amount = self.ingredient_amount.iter().map(String::deref);
        (recipe, ingredient_amount)
    }
}

impl axum::response::IntoResponse for &JsonRecipe {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::OK, axum::Json(&self)).into_response()
    }
}

pub async fn get(db: &SqlitePool, recipe_id: &str) -> Result<(Recipe, Vec<String>), sqlx::Error> {
    let recipe = sqlx::query_as!(Recipe, "SELECT * FROM recipes WHERE id = $1;", recipe_id)
        .fetch_one(db)
        .await?;

    let ingredient_amount: Vec<String> = sqlx::query_scalar!("SELECT ingredient_amount FROM ingredients WHERE recipe_id = $1;", recipe_id)
        .fetch_all(db)
        .await?;

    Ok((recipe, ingredient_amount))
}

pub async fn get_by_ingredients<'a, I>(db: &SqlitePool, ingredients: I) -> Result<Option<String>, sqlx::Error>
    where I: Iterator<Item=&'a str>
{
    let mut rtx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS qingredients;").execute(&mut *rtx).await?;
    sqlx::query("CREATE TEMPORARY TABLE qingredients (ingredient_amount VARCHR(200));")
        .execute(&mut *rtx)
        .await?;
    for ingredient_amount in ingredients {
        sqlx::query("INSERT INTO qingredients VALUES ($1);")
            .bind(ingredient_amount)
            .execute(&mut *rtx)
            .await?;
    }
    let recipe_ids = sqlx::query("SELECT DISTINCT recipe_id FROM ingredients JOIN qingredients ON ingredients.ingredient_amount = qingredients.ingredient_amount ORDER BY RANDOM() LIMIT 1;")
        .fetch_all(&mut *rtx)
        .await?;
    let nrecipe_ids = recipe_ids.len();
    let result = if nrecipe_ids == 1 {
        Some(recipe_ids[0].get(0))
    } else {
        None
    };
    rtx.commit().await?;

    Ok(result)
}

pub async fn get_random(db: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!("SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await
}
