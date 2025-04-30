mod error;
mod recipe;
mod templates;

use error::*;
use recipe::*;
use templates::*;

extern crate log;
extern crate mime;

use axum::{self, extract::State, response, routing};
use clap::Parser;
extern crate fastrand;
use sqlx::{SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

async fn get_recipe(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let mut app_state = app_state.write().await;
    let db = &app_state.db;
    let recipe = sqlx::query_as!(Recipe, "SELECT * FROM recipes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await;
    match recipe {
      Ok(recipe) => app_state.current_recipe = recipe,
      Err(e) => log::warn!("recipe fetch failed: {}", e),
    }
    let recipe = IndexTemplate::recipe(&app_state.current_recipe);
    response::Html(recipe.to_string())
}

fn get_db_uri(db_uri: Option<&str>) -> String {
    if let Some(db_uri) = db_uri {
        db_uri.to_string()
    } else if let Ok(db_uri) = std::env::var("DB_URI") {
        db_uri
    } else {
        "sqlite://db.db".to_string()
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
'next_recipe:
        for rr in recipes {
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
                let ingredient_insert = sqlx::query!(
                    "INSERT INTO ingredients (recipe_id, ingredient_amount) VALUES ($1, $2);",
                    r.id,
                    i,
                )
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
    let state = Arc::new(RwLock::new(AppState { db, current_recipe }));

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
            "/knock.css",
            services::ServeFile::new_with_mime("assets/static/knock.css", &mime::TEXT_CSS_UTF_8,),
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
