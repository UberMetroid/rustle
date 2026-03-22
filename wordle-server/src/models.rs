use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct TeamData {
    pub points: i32,
    pub players: u32,
    pub yesterday_total: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GlobalStats {
    pub yellow: TeamData,
    pub red: TeamData,
    pub green: TeamData,
    pub blue: TeamData,
    pub purple: TeamData,
    pub orange: TeamData,
    pub yesterday_winner: String,
    pub current_date: String,
    pub server_utc_timestamp: u64,
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self {
            yellow: TeamData::default(),
            red: TeamData::default(),
            green: TeamData::default(),
            purple: TeamData::default(),
            blue: TeamData::default(),
            orange: TeamData::default(),
            yesterday_winner: "none".to_string(),
            current_date: Utc::now().format("%Y-%m-%d").to_string(),
            server_utc_timestamp: Utc::now().timestamp_millis() as u64,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ScorePayload {
    pub player_id: String,
    pub team: String,
    pub points_delta: i32,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}
