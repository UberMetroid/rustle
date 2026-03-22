use axum::{
    extract::{Json, State},
};
use chrono::Utc;
use sqlx::Row;
use crate::models::{AppState, GlobalStats, TeamData, ScorePayload};

/// Retrieves the global statistics for all teams and the current game state.
pub async fn get_stats(State(state): State<AppState>) -> Json<GlobalStats> {
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

    Json(stats)
}

/// Handles submitting a new score for a player and updating team statistics.
/// Enforces point delta limits and handles team switching logic.
pub async fn submit_score(State(state): State<AppState>, Json(payload): Json<ScorePayload>) {
    // Basic validation of point delta
    if payload.points_delta < -5 || payload.points_delta > 10 {
        return;
    }

    let valid_teams = ["red", "orange", "yellow", "green", "blue", "purple"];
    if !valid_teams.contains(&payload.team.as_str()) {
        return;
    }

    if let Ok(mut tx) = state.pool.begin().await {
        // Check if player already exists and which team they were on
        let prev_team_opt: Option<String> =
            sqlx::query("SELECT team FROM players WHERE player_id = ?")
                .bind(&payload.player_id)
                .fetch_optional(&mut *tx)
                .await
                .ok()
                .flatten()
                .map(|r| r.get("team"));

        if prev_team_opt.as_deref() != Some(payload.team.as_str()) {
            // Player is joining a new team or switching teams
            if let Some(prev) = prev_team_opt {
                // Remove player from previous team count
                let _ =
                    sqlx::query("UPDATE teams SET players = MAX(0, players - 1) WHERE name = ?")
                        .bind(prev)
                        .execute(&mut *tx)
                        .await;
            }
            
            // Register or update player's team assignment
            let _ = sqlx::query("INSERT INTO players (player_id, team) VALUES (?, ?) ON CONFLICT(player_id) DO UPDATE SET team = ?")
                .bind(&payload.player_id).bind(&payload.team).bind(&payload.team)
                .execute(&mut *tx).await;

            // Increment new team player count
            let _ = sqlx::query("UPDATE teams SET players = players + 1 WHERE name = ?")
                .bind(&payload.team)
                .execute(&mut *tx)
                .await;
        }

        // Apply point delta to the team
        let _ = sqlx::query("UPDATE teams SET points = points + ? WHERE name = ?")
            .bind(payload.points_delta)
            .bind(&payload.team)
            .execute(&mut *tx)
            .await;

        let _ = tx.commit().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        db::run_migrations(&pool).await;
        db::seed_db(&pool).await;
        pool
    }

    #[tokio::test]
    async fn test_get_stats_empty() {
        let pool = setup_test_db().await;
        let state = AppState { pool };
        
        let response = get_stats(State(state)).await;
        let stats = response.0;
        
        assert_eq!(stats.yesterday_winner, "none");
        assert_eq!(stats.red.points, 0);
        assert_eq!(stats.blue.points, 0);
    }

    #[tokio::test]
    async fn test_submit_score_valid() {
        let pool = setup_test_db().await;
        let state = AppState { pool: pool.clone() };
        
        let payload = ScorePayload {
            player_id: "test-player".to_string(),
            team: "red".to_string(),
            points_delta: 5,
        };
        
        submit_score(State(state.clone()), Json(payload)).await;
        
        let stats = get_stats(State(state)).await.0;
        assert_eq!(stats.red.points, 5);
        assert_eq!(stats.red.players, 1);
    }

    #[tokio::test]
    async fn test_submit_score_invalid_delta() {
        let pool = setup_test_db().await;
        let state = AppState { pool: pool.clone() };
        
        let payload = ScorePayload {
            player_id: "test-player".to_string(),
            team: "red".to_string(),
            points_delta: 100, // Invalid
        };
        
        submit_score(State(state.clone()), Json(payload)).await;
        
        let stats = get_stats(State(state)).await.0;
        assert_eq!(stats.red.points, 0);
    }

    #[tokio::test]
    async fn test_team_switching() {
        let pool = setup_test_db().await;
        let state = AppState { pool: pool.clone() };
        
        // Join Red
        submit_score(State(state.clone()), Json(ScorePayload {
            player_id: "p1".to_string(),
            team: "red".to_string(),
            points_delta: 2,
        })).await;
        
        // Switch to Blue
        submit_score(State(state.clone()), Json(ScorePayload {
            player_id: "p1".to_string(),
            team: "blue".to_string(),
            points_delta: 3,
        })).await;
        
        let stats = get_stats(State(state)).await.0;
        assert_eq!(stats.red.players, 0);
        assert_eq!(stats.blue.players, 1);
        assert_eq!(stats.red.points, 2);
        assert_eq!(stats.blue.points, 3);
    }
}

