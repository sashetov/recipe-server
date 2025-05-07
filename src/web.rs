use crate::*;

#[derive(Deserialize)]
pub struct GetRecipeParams {
    id: Option<String>,
    ingredients: Option<String>,
}

async fn recipe_by_ingredients(db: &SqlitePool, ingredients: &str) -> Result<Option<String>, sqlx::Error> {
    let mut rtx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS qingredients;").execute(&mut *rtx).await?;
    sqlx::query("CREATE TEMPORARY TABLE qingredients (ingredient_amount VARCHR(200));")
        .execute(&mut *rtx)
        .await?;
    for ingredient_amount in ingredients.split(',') {
        sqlx::query("INSERT INTO qingredients VALUES ($1);")
            .bind(ingredient_amount)
            .execute(&mut *rtx)
            .await?;
    }
    let recipe_ids = sqlx::query("SELECT DISTINCT recipe_id FROM ingredients JOIN qingredients ON ingredients.ingredient_amount LIKE '%' || qingredients.ingredient_amount || '%' ORDER BY RANDOM() LIMIT 1;")
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

pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_state = app_state.write().await;
    let db = app_state.db.clone();


    if let GetRecipeParams { id: Some(id), .. } = params {
        let recipe_result = sqlx::query_as!(Recipe, "SELECT * FROM recipes WHERE id = $1;", id)
            .fetch_one(&db)
            .await;
        let result = match recipe_result {
            Ok(recipe) => {
                let mut ingredients =
                    sqlx::query_scalar!("SELECT ingredient_amount FROM ingredients WHERE recipe_id = $1;", recipe.id)
                        .fetch(&db);
                let mut ingredients_list: Vec<String> = Vec::new();
                while let Some(ingredient) = ingredients.next().await {
                    let ingredient = ingredient.unwrap_or_else(|e| {
                        log::error!("ingredient fetch failed: {}", e);
                        panic!("ingredient fetch failed")
                    });
                    ingredients_list.push(ingredient);
                }
                let ingredients_string = ingredients_list.join(", ");

                app_state.current_recipe = recipe.clone();
                let recipe = IndexTemplate::new(recipe.clone(), ingredients_string);
                Ok(response::Html(recipe.to_string()).into_response())
            }
            Err(e) => {
                log::warn!("recipe fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        };
        return result;
    }

    if let GetRecipeParams { ingredients: Some(ingredients), .. } = params {
        log::info!("recipe ingredients: {}", ingredients);

        let mut ingredients_string = String::new();
        for c in ingredients.chars() {
            if c.is_alphabetic() || c == ',' {
                let cl: String = c.to_lowercase().collect();
                ingredients_string.push_str(&cl);
            }
        }

        let recipe_result = recipe_by_ingredients(&db, &ingredients_string).await;
        match recipe_result {
            Ok(Some(id)) => {
                let uri = format!("/?id={}", id);
                return Ok(response::Redirect::to(&uri).into_response());
            }
            Ok(None) => {
                log::info!("recipe by ingredients selection was empty");
            }
            Err(e) => {
                log::error!("recipes by ingredients selection database error: {}", e);
                panic!("recipes by ingredients selection database error");
            }
        }
    }


    let recipe_result = sqlx::query_scalar!("SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(&db)
        .await;
    match recipe_result {
        Ok(id) => {
            let uri = format!("/?id={}", id);
            Ok(response::Redirect::to(&uri).into_response())
        }
        Err(e) => {
            log::error!("recipe selection failed: {}", e);
            panic!("recipe selection failed");
        }
    }
}
