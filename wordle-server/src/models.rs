use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;
use sqlx::SqlitePool;

/// Represents the statistics for a single team.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct TeamData {
    /// Total points accumulated by the team today.
    pub points: i32,
    /// Number of unique players currently assigned to this team.
    pub players: u32,
    /// The total points this team had at the end of the previous day.
    pub yesterday_total: i32,
}

/// The global state of the game, including all team scores and current date info.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GlobalStats {
    pub yellow: TeamData,
    pub red: TeamData,
    pub green: TeamData,
    pub blue: TeamData,
    pub purple: TeamData,
    pub orange: TeamData,
    /// The name of the team that won yesterday.
    pub yesterday_winner: String,
    /// A mapping of player unique IDs to their current team names.
    pub active_players: HashMap<String, String>,
    /// The current game date in YYYY-MM-DD format.
    pub current_date: String,
    /// Current server time in milliseconds since epoch.
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
            active_players: HashMap::new(),
            current_date: Utc::now().format("%Y-%m-%d").to_string(),
            server_utc_timestamp: Utc::now().timestamp_millis() as u64,
        }
    }
}

/// The request payload sent when a player submits their game score.
#[derive(Deserialize, Debug, Clone)]
pub struct ScorePayload {
    /// Unique identifier for the player.
    pub player_id: String,
    /// The team name the player is contributing to.
    pub team: String,
    /// The number of points to add (or subtract).
    pub points_delta: i32,
}

/// Shared application state containing the database connection pool.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}
