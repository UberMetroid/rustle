use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use chrono::Utc;

/// Initializes the SQLite database and ensures all required tables and default records exist.
pub async fn init_db(db_url: &str) -> SqlitePool {
    // Ensure data directory exists if using the default path
    if db_url.starts_with("sqlite:data/") {
        let _ = std::fs::create_dir_all("data");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to connect to SQLite DB");

    run_migrations(&pool).await;
    seed_db(&pool).await;

    pool
}

/// Runs the SQL schema migrations to create necessary tables.
pub async fn run_migrations(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS teams (
            name TEXT PRIMARY KEY,
            points INTEGER DEFAULT 0,
            players INTEGER DEFAULT 0,
            yesterday_total INTEGER DEFAULT 0
        );",
    )
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            current_date TEXT,
            yesterday_winner TEXT
        );",
    )
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS players (
            player_id TEXT PRIMARY KEY,
            team TEXT
        );",
    )
    .execute(pool)
    .await
    .unwrap();
}

/// Seeds the database with default team and system state records if they don't already exist.
pub async fn seed_db(pool: &SqlitePool) {
    // Ensure all teams exist
    for team in ["red", "orange", "yellow", "green", "blue", "purple"] {
        sqlx::query("INSERT OR IGNORE INTO teams (name, points, players, yesterday_total) VALUES (?, 0, 0, 0)")
            .bind(team)
            .execute(pool).await.unwrap();
    }

    // Ensure sys_state exists
    let today = Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query("INSERT OR IGNORE INTO sys_state (id, current_date, yesterday_winner) VALUES (1, ?, 'none')")
        .bind(&today)
        .execute(pool).await.unwrap();
}
