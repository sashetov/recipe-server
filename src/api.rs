use crate::*;
async fn get_recipe_by_id(db: &SqlitePool, recipe_id: &str) -> Result<response::Response, http::StatusCode> {
    let recipe_result = recipe::get(db, recipe_id).await;
    match recipe_result {
        Ok((recipe, ingredients)) => Ok(JsonRecipe::new(recipe, ingredients).into_response()),
        Err(e) => {
            log::warn!("recipe fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(recipe_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    get_recipe_by_id(db, &recipe_id).await
}

pub async fn get_recipe_by_ingredients(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(ingredients): Json<Vec<String>>,
) -> Result<response::Response, http::StatusCode> {
    log::info!("get recipe by ingredients: {:?}", ingredients);
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let recipe_result = recipe::get_by_ingredients(db, ingredients.iter().map(String::as_ref)).await;
    match recipe_result {
        Ok(Some(recipe_id)) => get_recipe_by_id(db, &recipe_id).await,
        Ok(None) => {
            log::warn!("recipe fetch by ingredients failed");
            Err(http::StatusCode::NOT_FOUND)
        }
        Err(e) => {
            log::warn!("recipe ingredients fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

pub async fn get_random_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let recipe_result = recipe::get_random(db).await;
    match recipe_result {
        Ok(recipe_id) => get_recipe_by_id(db, &recipe_id.to_string()).await,
        Err(e) => {
            log::warn!("get random recipe failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}
