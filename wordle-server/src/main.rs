use axum::{
    extract::{State, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use chrono::Utc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use std::net::SocketAddr;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct TeamData {
    pub points: i32,
    pub players: u32,
    pub yesterday_total: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GlobalStats {
    pub yellow: TeamData,
    pub red: TeamData,
    pub green: TeamData,
    pub blue: TeamData,
    pub orange: TeamData,
    pub yesterday_winner: String,
    pub active_players: HashMap<String, String>, // player_id -> team
    pub current_date: String,
    pub server_utc_timestamp: u64,
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self {
            yellow: TeamData::default(),
            red: TeamData::default(),
            green: TeamData::default(),
            blue: TeamData::default(),
            orange: TeamData::default(),
            yesterday_winner: "none".to_string(),
            active_players: HashMap::new(),
            current_date: Utc::now().format("%Y-%m-%d").to_string(),
            server_utc_timestamp: Utc::now().timestamp_millis() as u64,
        }
    }
}

#[derive(Deserialize)]
struct ScorePayload {
    player_id: String,
    team: String,
    points_delta: i32,
}

#[derive(Clone)]
struct AppState {
    pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv().ok();
    
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/wordle.db?mode=rwc".to_string());
    
    // Ensure data directory exists if using the default path
    if db_url.starts_with("sqlite:data/") {
        let _ = std::fs::create_dir_all("data");
    }

    // Initialize SQLite Database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url).await.expect("Failed to connect to SQLite DB");

    // Initialize Schema
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS teams (
            name TEXT PRIMARY KEY,
            points INTEGER DEFAULT 0,
            players INTEGER DEFAULT 0,
            yesterday_total INTEGER DEFAULT 0
        );"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            current_date TEXT,
            yesterday_winner TEXT
        );"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS players (
            player_id TEXT PRIMARY KEY,
            team TEXT
        );"
    ).execute(&pool).await.unwrap();

    // Ensure all teams exist
    for team in ["red", "orange", "yellow", "green", "blue"] {
        sqlx::query("INSERT OR IGNORE INTO teams (name, points, players, yesterday_total) VALUES (?, 0, 0, 0)")
            .bind(team)
            .execute(&pool).await.unwrap();
    }

    // Ensure sys_state exists
    let today = Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query("INSERT OR IGNORE INTO sys_state (id, current_date, yesterday_winner) VALUES (1, ?, 'none')")
        .bind(&today)
        .execute(&pool).await.unwrap();

    let state = AppState { pool: pool.clone() };

    // Background task for Midnight Rollover
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            let current_date = Utc::now().format("%Y-%m-%d").to_string();
            
            if let Ok(row) = sqlx::query("SELECT current_date FROM sys_state WHERE id = 1").fetch_one(&pool_clone).await {
                let db_date: String = row.get("current_date");
                if db_date != current_date {
                    if let Ok(winner_row) = sqlx::query("SELECT name FROM teams ORDER BY points DESC LIMIT 1").fetch_one(&pool_clone).await {
                        let winner: String = winner_row.get("name");
                        
                        if let Ok(mut tx) = pool_clone.begin().await {
                            let _ = sqlx::query("UPDATE sys_state SET current_date = ?, yesterday_winner = ? WHERE id = 1")
                                .bind(&current_date).bind(&winner).execute(&mut *tx).await;
                            
                            let _ = sqlx::query("UPDATE teams SET yesterday_total = points, points = 0, players = 0").execute(&mut *tx).await;
                            let _ = sqlx::query("DELETE FROM players").execute(&mut *tx).await;
                            
                            let _ = tx.commit().await;
                        }
                    }
                }
            }
        }
    });

    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_millisecond(500)
            .burst_size(10)
            .finish()
            .unwrap()
    );

    let app = Router::new()
        .route("/global-stats.json", get(get_stats))
        .route("/api/score", post(submit_score).layer(GovernorLayer { config: Box::leak(governor_conf) }))
        .fallback_service(ServeDir::new("dist"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7583").await.unwrap();
    println!("Listening on port 7583");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn get_stats(State(state): State<AppState>) -> Json<GlobalStats> {
    let mut stats = GlobalStats {
        server_utc_timestamp: Utc::now().timestamp_millis() as u64,
        ..Default::default()
    };

    if let Ok(row) = sqlx::query("SELECT current_date, yesterday_winner FROM sys_state WHERE id = 1").fetch_one(&state.pool).await {
        stats.current_date = row.get("current_date");
        stats.yesterday_winner = row.get("yesterday_winner");
    }

    if let Ok(rows) = sqlx::query("SELECT name, points, players, yesterday_total FROM teams").fetch_all(&state.pool).await {
        for row in rows {
            let name: String = row.get("name");
            let pts: i64 = row.get("points");
            let plyrs: i64 = row.get("players");
            let yt: i64 = row.get("yesterday_total");
            
            let t_data = TeamData {
                points: pts as i32,
                players: plyrs as u32,
                yesterday_total: yt as i32,
            };

            match name.as_str() {
                "red" => stats.red = t_data,
                "orange" => stats.orange = t_data,
                "yellow" => stats.yellow = t_data,
                "green" => stats.green = t_data,
                "blue" => stats.blue = t_data,
                _ => {}
            }
        }
    }
    
    Json(stats)
}

async fn submit_score(State(state): State<AppState>, Json(payload): Json<ScorePayload>) {
    if payload.points_delta < -5 || payload.points_delta > 10 {
        return;
    }
    
    let valid_teams = ["red", "orange", "yellow", "green", "blue"];
    if !valid_teams.contains(&payload.team.as_str()) {
        return;
    }

    if let Ok(mut tx) = state.pool.begin().await {
        let prev_team_opt: Option<String> = sqlx::query("SELECT team FROM players WHERE player_id = ?")
            .bind(&payload.player_id)
            .fetch_optional(&mut *tx).await.ok().flatten().map(|r| r.get("team"));

        if prev_team_opt.as_deref() != Some(payload.team.as_str()) {
            if let Some(prev) = prev_team_opt {
                let _ = sqlx::query("UPDATE teams SET players = MAX(0, players - 1) WHERE name = ?")
                    .bind(prev)
                    .execute(&mut *tx).await;
            }
            let _ = sqlx::query("INSERT INTO players (player_id, team) VALUES (?, ?) ON CONFLICT(player_id) DO UPDATE SET team = ?")
                .bind(&payload.player_id).bind(&payload.team).bind(&payload.team)
                .execute(&mut *tx).await;
            
            let _ = sqlx::query("UPDATE teams SET players = players + 1 WHERE name = ?")
                .bind(&payload.team)
                .execute(&mut *tx).await;
        }

        let _ = sqlx::query("UPDATE teams SET points = points + ? WHERE name = ?")
            .bind(payload.points_delta).bind(&payload.team)
            .execute(&mut *tx).await;

        let _ = tx.commit().await;
    }
}
