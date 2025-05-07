mod error;
mod recipe;
mod templates;

use error::*;
use recipe::*;
use templates::*;

extern crate log;
extern crate mime;

use axum::{
    self,
    extract::{Query, State},
    http,
    response::{self, IntoResponse},
    routing,
};
use clap::Parser;
extern crate fastrand;
use serde::Deserialize;
use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tokio_stream::StreamExt;
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::borrow::Cow;
use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
}

struct AppState {
    db: SqlitePool,
    current_recipe: Recipe,
}
#[derive(Deserialize)]
struct GetRecipeParams {
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

async fn get_recipe(
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

fn get_db_uri(db_uri: Option<&str>) -> Cow<str> {
    if let Some(db_uri) = db_uri {
        db_uri.into()
    } else if let Ok(db_uri) = std::env::var("DB_URI") {
        db_uri.into()
    } else {
        "sqlite://db.db".into()
    }
}

fn extract_db_dir(db_uri: &str) -> Result<&str, RecipeError> {
    if db_uri.starts_with("sqlite://") && db_uri.ends_with(".db") {
        let start = db_uri.find(':').unwrap() + 3;
        let mut path = &db_uri[start..];
        if let Some(end) = path.rfind('/') {
            path = &path[..end];
        } else {
            path = "";
        }
        Ok(path)
    } else {
        Err(RecipeError::InvalidDbUri(db_uri.to_string()))
    }
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db_uri = get_db_uri(args.db_uri.as_deref());
    if !sqlite::Sqlite::database_exists(&db_uri).await? {
        let db_dir = extract_db_dir(&db_uri)?;
        std::fs::create_dir_all(db_dir)?;
        sqlite::Sqlite::create_database(&db_uri).await?
    }

    let db = SqlitePool::connect(&db_uri).await?;
    sqlx::migrate!().run(&db).await?;
    if let Some(path) = args.init_from {
        let recipes = read_recipes(path)?;
        'next_recipe: for rr in recipes {
            let mut rtx = db.begin().await?;
            let (r, is) = rr.to_recipe();
            let recipe_insert = sqlx::query!(
                "INSERT INTO recipes (id, title, category, preparation) VALUES ($1, $2, $3, $4);",
                r.id,
                r.title,
                r.category,
                r.preparation,
            )
            .execute(&mut *rtx)
            .await;
            if let Err(e) = recipe_insert {
                eprintln!("error: recipe insert: {}: {}", r.id, e);
                rtx.rollback().await?;
                continue;
            };
            for i in is {
                let ingredient_insert = 
                    sqlx::query!("INSERT INTO ingredients (recipe_id, ingredient_amount) VALUES ($1, $2);", r.id, i,)
                        .execute(&mut *rtx)
                        .await;
                if let Err(e) = ingredient_insert {
                    eprintln!("error: ingredient insert: {} {}: {}", r.id, i, e);
                    rtx.rollback().await?;
                    continue 'next_recipe;
                };
            }
            rtx.commit().await?;
        }
    }
    let current_recipe = Recipe {
        id: 0,
        title: "thing".to_string(),
        category: "thingies".to_string(),
        preparation: "notreal".to_string(),
    };
    let app_state = AppState { db, current_recipe };
    let state = Arc::new(RwLock::new(app_state));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "recipes-server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();
    let app = axum::Router::new()
        .route("/", routing::get(get_recipe))
        .route_service(
            "/style.css",
            services::ServeFile::new_with_mime("assets/static/style.css", &mime::TEXT_CSS_UTF_8,),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime( "assets/static/favicon.ico", &mime_favicon,),
        )
        .layer(trace_layer)
        .with_state(state);

    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("recipes-server: error: {}", err);
        std::process::exit(1);
    }
}
