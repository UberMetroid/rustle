use std::net::SocketAddr;
use wordle_server::models::AppState;
use wordle_server::{create_router, db, spawn_rollover_task};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/wordle.db?mode=rwc".to_string());

    let pool = db::init_db(&db_url).await;
    let state = AppState { pool: pool.clone() };

    // Start background task for midnight rollover
    spawn_rollover_task(pool.clone());

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7583")
        .await
        .expect("Failed to bind to port 7583");
    println!("Listening on port 7583");
    
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Server failed to start");
}
