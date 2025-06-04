use crate::*;

#[derive(Deserialize)]
pub struct GetRecipeParams {
    id: Option<String>,
    ingredients: Option<String>,
}

pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_writer = app_state.write().await;
    let db = app_writer.db.clone();

    // Specified.
    if let GetRecipeParams { id: Some(id), .. } = params {
        let recipe_result = recipe::get(&db, &id).await;
        let result = match recipe_result {
            Ok((recipe, ingredients)) => {
                let ingredients_string = ingredients.join(", ");

                app_writer.current_recipe = recipe.clone();
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

        let recipe_result = recipe::get_by_ingredients(&db, ingredients_string.split(',')).await;
        match recipe_result {
            Ok(Some(id)) => {
                let uri = format!("/?id={}", id);
                return Ok(response::Redirect::to(&uri).into_response());
            }
            Ok(None) => {
                log::info!("recipe by ingredietns selection was empty");
            }
            Err(e) => {
                log::error!("recipe by ingredients selection database error: {}", e);
                panic!("recipe by ingredients selection database error");
            }
        }
    }

    let recipe_result = recipe::get_random(&db).await;
    match recipe_result {
        Ok(id) => {
            let uri = format!("/?id={}", id.to_string());
            Ok(response::Redirect::to(&uri).into_response())
        }
        Err(e) => {
            log::error!("recipe selection failed: {}", e);
            let ingredient_string = "empty".to_string();
            let recipe = app_writer.current_recipe.clone();
            let recipe = IndexTemplate::new(recipe, ingredient_string);
            Ok(response::Html(recipe.to_string()).into_response())
        }
    }
}
