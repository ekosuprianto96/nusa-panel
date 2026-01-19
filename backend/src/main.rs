//! # NusaPanel Main Entry Point
//!
//! Entry point untuk aplikasi NusaPanel API.
//! Menginisialisasi Rocket server dengan semua routes dan middleware.

#[macro_use]
extern crate rocket;

use dotenv::dotenv;
use rocket::fairing::AdHoc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use nusa_panel::config::AppConfig;
use nusa_panel::database::Database;
use nusa_panel::routes;
use rocket::fs::{FileServer, NamedFile};
use std::path::PathBuf;

/// Fallback route untuk SPA (Single Page Application)
/// 
/// Mengembalikan index.html untuk semua route yang tidak match dengan API atau static files.
/// Ini memungkinkan Vue Router untuk handle routing di client-side.
#[get("/<_path..>", rank = 100)]
async fn spa_fallback(_path: PathBuf) -> Option<NamedFile> {
    let dist_dir = std::env::var("FRONTEND_DIST")
        .unwrap_or_else(|_| "frontend/dist".to_string());

    let index_path = PathBuf::from(dist_dir).join("index.html");

    NamedFile::open(index_path).await.ok()
}

/// Konfigurasi CORS untuk keamanan cross-origin requests
fn configure_cors() -> rocket_cors::CorsOptions {
    rocket_cors::CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::all(),
        allowed_methods: vec![
            rocket::http::Method::Get,
            rocket::http::Method::Post,
            rocket::http::Method::Put,
            rocket::http::Method::Delete,
            rocket::http::Method::Options,
        ]
        .into_iter()
        .map(From::from)
        .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
}

/// Inisialisasi tracing/logging
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nusa_panel=debug,rocket=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Build dan launch Rocket server
///
/// # Returns
/// Rocket instance yang siap di-launch
#[launch]
async fn rocket() -> _ {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    init_tracing();

    tracing::info!("🚀 Starting NusaPanel API Server...");

    // Load application configuration
    let config = AppConfig::from_env();

    // Setup CORS
    let cors = configure_cors()
        .to_cors()
        .expect("Failed to create CORS fairing");

    let frontend_path = std::env::var("FRONTEND_DIST")
    .unwrap_or_else(|_| format!("{}/frontend/dist", std::env::current_dir().unwrap().display()));

    // Build Rocket instance
    rocket::build()
        // Attach CORS
        .attach(cors)
        // Attach database connection pool
        .attach(Database::fairing())
        // Attach application config
        // Attach application config
        // .attach(AdHoc::config::<AppConfig>())
        // Mount routes
        // Mount routes
        .mount("/api", routes::health_routes())
        .mount("/api/auth", routes::auth_routes())
        .mount("/api/users", routes::user_routes())
        .mount("/api/domains", routes::domain_routes())
        .mount("/api/files", routes::file_routes())
        .mount("/api/ftp", routes::ftp_routes())
        .mount("/api/databases", routes::database_routes())
        .mount("/api/emails", routes::email_routes())
        .mount("/api/web-server", routes::web_server_routes())
        .mount("/api/security", routes::security_routes())
        .mount("/api/system", routes::system_routes())
        .mount("/api/apps", routes::app_routes())
        .mount("/api/redis", routes::redis_routes())
        .mount("/api/nodejs", routes::nodejs_routes())
        // Serve Static Files for Frontend
        .mount("/", FileServer::from(&frontend_path))
        // Frontend
        .mount("/", routes![spa_fallback])
        // Manage application state
        .manage(config)
}
