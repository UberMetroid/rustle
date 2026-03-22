use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use sqlx::Row;
use crate::models::{AppState, GlobalStats, TeamData, ScorePayload};

const VALID_TEAMS: &[&str] = &["red", "orange", "yellow", "green", "blue", "purple"];

fn is_valid_team(team: &str) -> bool {
    VALID_TEAMS.contains(&team)
}

pub async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let mut stats = GlobalStats {
        server_utc_timestamp: Utc::now().timestamp_millis() as u64,
        ..Default::default()
    };

    if let Ok(row) =
        sqlx::query("SELECT current_date, yesterday_winner FROM sys_state WHERE id = 1")
            .fetch_one(&state.pool)
            .await
    {
        stats.current_date = row.get("current_date");
        stats.yesterday_winner = row.get("yesterday_winner");
    }

    if let Ok(rows) = sqlx::query("SELECT name, points, players, yesterday_total FROM teams")
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
                _ => {}
            }
        }
    }

    (StatusCode::OK, Json(stats))
}

pub async fn submit_score(
    State(state): State<AppState>,
    Json(payload): Json<ScorePayload>,
) -> impl IntoResponse {
    if payload.player_id.is_empty() || payload.player_id.len() > 64 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid player_id"})));
    }

    if payload.player_id.chars().any(|c| !c.is_alphanumeric() && c != '_' && c != '-') {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid player_id characters"})));
    }

    let team = payload.team.to_lowercase();
    if !is_valid_team(&team) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid team"})));
    }

    if payload.points_delta < -5 || payload.points_delta > 10 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Points delta out of range"})));
    }

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"}))),
    };

    let prev_team_opt: Option<String> = sqlx::query("SELECT team FROM players WHERE player_id = ?")
        .bind(&payload.player_id)
        .fetch_optional(&mut *tx)
        .await
        .ok()
        .flatten()
        .map(|r| r.get("team"));

    if prev_team_opt.as_deref() != Some(&team) {
        if let Some(prev) = prev_team_opt {
            if sqlx::query("UPDATE teams SET players = MAX(0, players - 1) WHERE name = ?")
                .bind(prev)
                .execute(&mut *tx)
                .await
                .is_err()
            {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"})));
            }
        }
        
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
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"})));
        }

        if sqlx::query("UPDATE teams SET players = players + 1 WHERE name = ?")
            .bind(&team)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"})));
        }
    }

    if sqlx::query("UPDATE teams SET points = points + ? WHERE name = ?")
        .bind(payload.points_delta)
        .bind(&team)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"})));
    }

    if tx.commit().await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Commit failed"})));
    }

    (StatusCode::OK, Json(serde_json::json!({"success": true})))
}
