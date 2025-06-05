mod cookie;
mod finder;
mod recipe;

use cookie::*;
use finder::*;
use recipe::*;

use std::collections::HashSet;

extern crate serde;
// use gloo_console::log;
use gloo_net::http;
extern crate wasm_bindgen_futures;
use wasm_cookies as cookies;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

pub type RecipeResult = Result<RecipeStruct, gloo_net::Error>;

struct App {
    cookie: String,
    recipe: RecipeResult,
}

pub enum Msg {
    GotRecipe(RecipeResult),
    GetRecipe(Option<String>),
}

impl App {
    fn refresh_recipe(ctx: &Context<Self>, key: Option<String>) {
        let got_recipe = RecipeStruct::get_recipe(key);
        ctx.link().send_future(got_recipe);
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let cookie = acquire_cookie();
        App::refresh_recipe(ctx, None);
        let recipe = Err(gloo_net::Error::GlooError("Loading Recipe…".to_string()));
        Self { cookie, recipe }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GotRecipe(recipe) => {
                self.recipe = recipe;
                true
            }
            Msg::GetRecipe(key) => {
                // log!(format!("GetRecipe: {:?}", key));
                App::refresh_recipe(ctx, key);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cookie = &self.cookie;
        let recipe = &self.recipe;
        html! {
        <>
            <h1>{ "Recipes" }</h1>
            if false {
                {render_cookie(cookie)}
            }
            if let Ok(recipe) = recipe {
                <Recipe recipe={recipe.clone()}/>
            }
            if let Err(error) = recipe {
                <div>
                    <span class="error">{format!("Server Error: {error}")}</span>
                </div>
            }
            <div>
                <button onclick={ctx.link().callback(|_| Msg::GetRecipe(None))}>{"Get another recipe"}</button>
            </div>
            <Finder on_find={ctx.link().callback(Msg::GetRecipe)}/>
        </>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
