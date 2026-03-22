use axum::{
    extract::{State, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use chrono::Utc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(Default)]
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

type AppState = Arc<RwLock<GlobalStats>>;

#[tokio::main]
async fn main() {
    let state_file = "global-stats.json";
    
    let initial_state = if let Ok(data) = std::fs::read_to_string(state_file) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        GlobalStats::default()
    };

    let state = Arc::new(RwLock::new(initial_state));

    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            let current_date = Utc::now().format("%Y-%m-%d").to_string();
            let mut w = state_clone.write().await;
            
            if w.current_date != current_date {
                let mut max_pts = w.yellow.points;
                let mut winner = "yellow".to_string();
                
                let teams = vec![
                    ("red", w.red.points),
                    ("green", w.green.points),
                    ("blue", w.blue.points),
                    ("orange", w.orange.points),
                ];
                for (name, pts) in teams {
                    if pts > max_pts { max_pts = pts; winner = name.to_string(); }
                }
                
                w.yesterday_winner = winner;
                w.yellow.yesterday_total = w.yellow.points; w.yellow.points = 0; w.yellow.players = 0;
                w.red.yesterday_total = w.red.points; w.red.points = 0; w.red.players = 0;
                w.green.yesterday_total = w.green.points; w.green.points = 0; w.green.players = 0;
                w.blue.yesterday_total = w.blue.points; w.blue.points = 0; w.blue.players = 0;
                w.orange.yesterday_total = w.orange.points; w.orange.points = 0; w.orange.players = 0;
                
                w.active_players.clear();
                w.current_date = current_date;
            }
            
            if let Ok(data) = serde_json::to_string(&*w) {
                let _ = std::fs::write("global-stats.json", data);
            }
        }
    });

    let app = Router::new()
        .route("/global-stats.json", get(get_stats))
        .route("/api/score", post(submit_score))
        .fallback_service(ServeDir::new("dist"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7583").await.unwrap();
    println!("Listening on port 7583");
    axum::serve(listener, app).await.unwrap();
}

async fn get_stats(State(state): State<AppState>) -> Json<GlobalStats> {
    let mut response = state.read().await.clone();
    response.server_utc_timestamp = Utc::now().timestamp_millis() as u64;
    Json(response)
}

async fn submit_score(State(state): State<AppState>, Json(payload): Json<ScorePayload>) {
    let mut w = state.write().await;
    
    let previous_team = w.active_players.insert(payload.player_id.clone(), payload.team.clone());
    
    if previous_team.as_deref() != Some(payload.team.as_str()) {
        if let Some(prev) = previous_team {
            match prev.as_str() {
                "yellow" => { w.yellow.players = w.yellow.players.saturating_sub(1); }
                "red" => { w.red.players = w.red.players.saturating_sub(1); }
                "green" => { w.green.players = w.green.players.saturating_sub(1); }
                "blue" => { w.blue.players = w.blue.players.saturating_sub(1); }
                "orange" => { w.orange.players = w.orange.players.saturating_sub(1); }
                _ => {}
            }
        }
        match payload.team.as_str() {
            "yellow" => { w.yellow.players += 1; }
            "red" => { w.red.players += 1; }
            "green" => { w.green.players += 1; }
            "blue" => { w.blue.players += 1; }
            "orange" => { w.orange.players += 1; }
            _ => {}
        }
    }

    match payload.team.as_str() {
        "yellow" => { w.yellow.points += payload.points_delta; }
        "red" => { w.red.points += payload.points_delta; }
        "green" => { w.green.points += payload.points_delta; }
        "blue" => { w.blue.points += payload.points_delta; }
        "orange" => { w.orange.points += payload.points_delta; }
        _ => {}
    }
}
