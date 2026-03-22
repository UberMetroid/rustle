//! HTTP Request Handlers
//! 
//! This module contains all API endpoint handlers for the Rustle server.

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use sqlx::Row;
use tracing::{error, info, warn};
use crate::models::{AppState, GlobalStats, TeamData, ScorePayload};

/// Valid team names
const VALID_TEAMS: &[&str] = &["red", "orange", "yellow", "green", "blue", "purple"];

/// Maximum player ID length
const MAX_PLAYER_ID_LENGTH: usize = 64;

/// Minimum points delta (penalty)
const MIN_POINTS_DELTA: i32 = -5;

/// Maximum points delta (bonus)
const MAX_POINTS_DELTA: i32 = 10;

/// Checks if a team name is valid (case-insensitive)
fn is_valid_team(team: &str) -> bool {
    VALID_TEAMS.contains(&team)
}

/// Retrieves the global statistics for all teams and current game state.
pub async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let mut stats = GlobalStats {
        server_utc_timestamp: Utc::now().timestamp_millis() as u64,
        ..Default::default()
    };

    // Fetch system state
    if let Ok(row) = sqlx::query(
        "SELECT current_date, yesterday_winner FROM sys_state WHERE id = 1"
    )
    .fetch_one(&state.pool)
    .await
    {
        stats.current_date = row.get("current_date");
        stats.yesterday_winner = row.get("yesterday_winner");
    }

    // Fetch all team data
    if let Ok(rows) = sqlx::query(
        "SELECT name, points, players, yesterday_total FROM teams"
    )
    .fetch_all(&state.pool)
    .await
    {
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
                "purple" => stats.purple = t_data,
                _ => warn!(event = "unknown_team", name = %name, "Unknown team in database")
            }
        }
    }

    info!(
        event = "stats_fetched",
        teams = 6,
        "Global stats retrieved"
    );

    (StatusCode::OK, Json(stats))
}

/// Handles score submission from players.
/// Validates input, updates team scores, and tracks player team membership.
pub async fn submit_score(
    State(state): State<AppState>,
    Json(payload): Json<ScorePayload>,
) -> impl IntoResponse {
    // Validate player_id length
    if payload.player_id.is_empty() || payload.player_id.len() > MAX_PLAYER_ID_LENGTH {
        warn!(
            event = "validation_failed",
            reason = "invalid_player_id_length",
            length = payload.player_id.len(),
            "Invalid player_id length"
        );
        return bad_request("Invalid player_id length (1-64 chars)");
    }

    // Validate player_id characters (alphanumeric, underscore, hyphen)
    if payload.player_id.chars().any(|c| !c.is_alphanumeric() && c != '_' && c != '-') {
        warn!(
            event = "validation_failed",
            reason = "invalid_player_id_chars",
            player_id = %payload.player_id,
            "Invalid player_id characters"
        );
        return bad_request("Invalid player_id characters");
    }

    // Normalize and validate team
    let team = payload.team.to_lowercase();
    if !is_valid_team(&team) {
        warn!(
            event = "validation_failed",
            reason = "invalid_team",
            team = %payload.team,
            "Invalid team name"
        );
        return bad_request("Invalid team");
    }

    // Validate points delta range
    if payload.points_delta < MIN_POINTS_DELTA || payload.points_delta > MAX_POINTS_DELTA {
        warn!(
            event = "validation_failed",
            reason = "invalid_points_delta",
            delta = payload.points_delta,
            range = format!("[{}, {}]", MIN_POINTS_DELTA, MAX_POINTS_DELTA),
            "Points delta out of range"
        );
        return bad_request(format!("Points delta must be between {} and {}", MIN_POINTS_DELTA, MAX_POINTS_DELTA));
    }

    // Start database transaction
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(event = "db_error", error = %e, "Failed to begin transaction");
            return internal_error("Database error");
        }
    };

    // Get previous team membership
    let prev_team_opt: Option<String> = sqlx::query(
        "SELECT team FROM players WHERE player_id = ?"
    )
    .bind(&payload.player_id)
    .fetch_optional(&mut *tx)
    .await
    .ok()
    .flatten()
    .map(|r| r.get("team"));

    // Handle team switching
    if prev_team_opt.as_deref() != Some(&team) {
        // Remove from previous team
        if let Some(prev) = prev_team_opt {
            if sqlx::query("UPDATE teams SET players = MAX(0, players - 1) WHERE name = ?")
                .bind(&prev)
                .execute(&mut *tx)
                .await
                .is_err()
            {
                error!(event = "db_error", error = "Failed to update previous team");
                return internal_error("Database error");
            }
            info!(
                event = "team_switch",
                player_id = %payload.player_id,
                from = %prev,
                to = %team,
                "Player switched teams"
            );
        }
        
        // Insert/update player team membership
        if sqlx::query(
            "INSERT INTO players (player_id, team) VALUES (?, ?) 
             ON CONFLICT(player_id) DO UPDATE SET team = ?"
        )
        .bind(&payload.player_id)
        .bind(&team)
        .bind(&team)
        .execute(&mut *tx)
        .await
        .is_err()
        {
            error!(event = "db_error", error = "Failed to update player team");
            return internal_error("Database error");
        }

        // Add to new team
        if sqlx::query("UPDATE teams SET players = players + 1 WHERE name = ?")
            .bind(&team)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            error!(event = "db_error", error = "Failed to increment team players");
            return internal_error("Database error");
        }
    }

    // Update team points
    if sqlx::query("UPDATE teams SET points = points + ? WHERE name = ?")
        .bind(payload.points_delta)
        .bind(&team)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        error!(event = "db_error", error = "Failed to update team points");
        return internal_error("Database error");
    }

    // Commit transaction
    if tx.commit().await.is_err() {
        error!(event = "db_error", error = "Failed to commit transaction");
        return internal_error("Commit failed");
    }

    info!(
        event = "score_submitted",
        player_id = %payload.player_id,
        team = %team,
        points_delta = payload.points_delta,
        "Score submitted successfully"
    );

    (StatusCode::OK, Json(serde_json::json!({"success": true})))
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": message.into()})))
}

fn internal_error(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": message.into()})))
}
