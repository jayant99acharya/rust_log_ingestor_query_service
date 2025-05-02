mod models;
mod handlers;
mod db;
mod error;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("Starting log ingestor service...");

    // Get database URL from environment variable
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    info!("Connecting to database...");

    // Initialize database
    let db = db::Database::new(&database_url)
        .await
        .expect("Failed to connect to database");
    
    info!("Initializing database schema...");

    db.init()
        .await
        .expect("Failed to initialize database");

    info!("Starting HTTP server on port 3000...");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(
                web::scope("/api")
                    .route("/logs", web::post().to(handlers::ingest_log))
                    .route("/logs/query", web::post().to(handlers::query_logs))
            )
            .route("/", web::get().to(handlers::ui::index))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
} 