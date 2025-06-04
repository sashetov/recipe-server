mod error;
mod recipe;
mod templates;
mod web;
mod api;
mod authjwt;

use error::*;
use recipe::*;
use templates::*;

extern crate log;
extern crate mime;

use axum::{
    self,
    RequestPartsExt,
    extract::{Path, Query, State, Json},
    http::{self, StatusCode},
    response::{self, IntoResponse},
    routing,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{prelude::*, TimeDelta};
use clap::Parser;
extern crate fastrand;
use jsonwebtoken::{EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, signal, sync::RwLock, time::Duration};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use std::borrow::Cow;
use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
    #[arg(short, long, default_value = "127.0.0.1")]
    ip: String,
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

struct AppState {
    db: SqlitePool,
    jwt_keys: authjwt::JwtKeys,
    reg_key: String,
    current_recipe: Recipe,
}

type SharedAppState = Arc<RwLock<AppState>>;

impl AppState {
    pub fn new(db: SqlitePool, jwt_keys: authjwt::JwtKeys, reg_key: String) -> Self {
        let current_recipe = Recipe {
            id: 0,
            title: "thing".to_string(),
            category: "thingies".to_string(),
            preparation: "notreal".to_string(),
        };
        Self {
            db,
            jwt_keys,
            reg_key,
            current_recipe,
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

// Thanks to Gemini for this code.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to create SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C (SIGINT) signal.");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal.");
        },
    }

    tracing::info!("Initiating graceful shutdown...");

    // Example: Give some time for in-flight requests to complete
    tokio::time::sleep(Duration::from_secs(2)).await;
    tracing::info!("Cleanup complete.");
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let tsf = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr);
    let tse = tracing_subscriber::EnvFilter::try_from_default_env().
        unwrap_or_else(|_| "rs=debug".into());
    tracing_subscriber::registry().with(tsf).with(tse).init();

    log::info!("Starting...");

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

    let jwt_keys = authjwt::make_jwt_keys().await.unwrap_or_else(|_| {
        tracing::error!("jwt keys");
        eprintln!("jwt keys err");
        std::process::exit(1);
    });

    let reg_key = authjwt::read_secret("REG_PASSWORD", "secrets/password.txt")
        .await
        .unwrap_or_else(|_| {
            tracing::error!("reg password");
            eprintln!("reg password");
            std::process::exit(1);
        });

    let app_state = AppState::new(db, jwt_keys, reg_key);
    let state = Arc::new(RwLock::new(app_state));

    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_origin(tower_http::cors::Any);

    async fn handler_404() -> axum::response::Response {
        (http::StatusCode::NOT_FOUND, "404 Not Found").into_response()
    }

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

    let (api_router, api) = OpenApiRouter::with_openapi(api::ApiDoc::openapi())
        .nest("/api/v1", api::router())
        .split_for_parts();

    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api.clone());
    let redoc_ui = Redoc::with_url("/redoc", api);
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    let app = axum::Router::new()
        .route("/", routing::get(web::get_recipe))
        .route_service(
            "/style.css",
            services::ServeFile::new_with_mime("assets/static/style.css", &mime::TEXT_CSS_UTF_8,),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime( "assets/static/favicon.ico", &mime_favicon,),
        )
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .merge(api_router)
        .fallback(handler_404)
        .layer(cors)
        .layer(trace_layer)
        .with_state(state);

    let endpoint = format!("{}:{}", args.ip, args.port);
    let listener = net::TcpListener::bind(&endpoint).await?;
    log::info!("started: listening on {}", endpoint);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}


#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("recipes-server: error: {}", err);
        std::process::exit(1);
    }
}
