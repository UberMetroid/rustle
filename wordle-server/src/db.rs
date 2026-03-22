use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use chrono::Utc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Failed to connect to database: {0}")]
    ConnectionError(String),
    #[error("Migration failed: {0}")]
    MigrationError(String),
    #[error("Seed failed: {0}")]
    SeedError(String),
}

pub async fn init_db(db_url: &str) -> Result<SqlitePool, DbError> {
    if db_url.starts_with("sqlite:data/") {
        let _ = std::fs::create_dir_all("data");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .map_err(|e| DbError::ConnectionError(e.to_string()))?;

    run_migrations(&pool).await?;
    seed_db(&pool).await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS teams (
            name TEXT PRIMARY KEY,
            points INTEGER DEFAULT 0,
            players INTEGER DEFAULT 0,
            yesterday_total INTEGER DEFAULT 0
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::MigrationError(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            current_date TEXT NOT NULL,
            yesterday_winner TEXT NOT NULL DEFAULT 'none'
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::MigrationError(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS players (
            player_id TEXT PRIMARY KEY,
            team TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::MigrationError(e.to_string()))?;

    Ok(())
}

pub async fn seed_db(pool: &SqlitePool) -> Result<(), DbError> {
    for team in ["red", "orange", "yellow", "green", "blue", "purple"] {
        sqlx::query(
            "INSERT OR IGNORE INTO teams (name, points, players, yesterday_total) 
             VALUES (?, 0, 0, 0)"
        )
        .bind(team)
        .execute(pool)
        .await
        .map_err(|e| DbError::SeedError(e.to_string()))?;
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query(
        "INSERT OR IGNORE INTO sys_state (id, current_date, yesterday_winner) 
         VALUES (1, ?, 'none')"
    )
    .bind(&today)
    .execute(pool)
    .await
    .map_err(|e| DbError::SeedError(e.to_string()))?;

    Ok(())
}
