use crate::*;

#[derive(Properties, Clone, PartialEq, serde::Deserialize)]
pub struct RecipeStruct {
    pub id: i64,
    pub title: String,
    pub category: String,
    pub ingredient_amount: Option<HashSet<String>>,
    pub preparation: Option<String>,
}

impl RecipeStruct {
    pub async fn get_recipe(key: Option<String>) -> Msg {
        let request = match &key {
            None => "http://localhost:3000/api/v1/random-recipe".to_string(),
            Some(key) => format!("http://localhost:3000/api/v1/recipe/{}", key,),
        };
        let response = http::Request::get(&request).send().await;
        match response {
            Err(e) => Msg::GotRecipe(Err(e)),
            Ok(data) => Msg::GotRecipe(data.json().await),
        }
    }
}
pub fn format_ingredients(ingredients: &HashSet<String>) -> String {
    let ingredients_list: Vec<&str> = ingredients.iter().map(String::as_ref).collect();
    ingredients_list.join(", ")
}

#[derive(Properties, Clone, PartialEq, serde::Deserialize)]
pub struct RecipeProps {
    pub recipe: RecipeStruct,
}

#[function_component(Recipe)]
pub fn recipe(recipe: &RecipeProps) -> Html {
    let recipe = &recipe.recipe;
    html! { <>
        <div class="recipe">
            <span class="bold">{recipe.title.clone()}</span><br/>
            <span>{recipe.category.clone()}</span><br/>
            <span>{recipe.preparation.clone().unwrap_or_else(|| "No preparation details available".to_string())}</span><br/>
        </div>
        <span class="annotation">
            {format!("[id: {}", &recipe.id)}
            if let Some(ref ingredients) = recipe.ingredient_amount{
                {format!("; ingredients: {}", &format_ingredients(ingredients))}
            }
            {"]"}
        </span>
    </> }
}
