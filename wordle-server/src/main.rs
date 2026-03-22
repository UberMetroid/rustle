//! Rustle Server Entry Point
//! 
//! This binary starts the HTTP server with all routes and background tasks.

use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use wordle_server::models::AppState;
use wordle_server::{create_router, db, spawn_rollover_task};

#[tokio::main]
async fn main() {
    // Initialize logging
    let log_level = std::env::var("RUST_LOG")
        .map(|l| l.parse::<Level>().unwrap_or(Level::INFO))
        .unwrap_or(Level::INFO);
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    let _ = dotenvy::dotenv().ok();

    // Load configuration from environment
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/wordle.db?mode=rwc".to_string());

    let dist_path = std::env::var("DIST_PATH")
        .unwrap_or_else(|_| "../wordle-ui/dist".to_string());

    let api_key_status = if std::env::var("API_KEY").is_ok() {
        "enabled"
    } else {
        "disabled (development mode)"
    };

    info!(
        event = "server_starting",
        port = 7583,
        dist_path = %dist_path,
        api_key_auth = %api_key_status,
        "Starting Rustle server"
    );

    let pool = db::init_db(&db_url)
        .await
        .expect("Failed to initialize database");
    
    let state = AppState { pool: pool.clone() };

    spawn_rollover_task(pool);

    let app = create_router(state, &dist_path);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7583")
        .await
        .expect("Failed to bind to port 7583");
    
    info!(
        event = "server_ready",
        addr = %listener.local_addr().unwrap(),
        "Server listening on port 7583"
    );
    
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Server failed to start");
}
