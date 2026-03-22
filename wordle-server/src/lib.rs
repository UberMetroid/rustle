pub mod models;
pub mod db;
pub mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use chrono::Utc;
use sqlx::Row;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::models::AppState;
use crate::handlers::{get_stats, submit_score};

/// Creates the Axum router with all routes, state, and middleware configured.
pub fn create_router(state: AppState) -> Router {
    let governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(500)
            .burst_size(10)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap_or_else(|| {
                GovernorConfigBuilder::default()
                    .key_extractor(SmartIpKeyExtractor)
                    .finish()
                    .unwrap()
            }),
    );

    Router::new()
        .route("/global-stats.json", get(get_stats))
        .route(
            "/api/score",
            post(submit_score).layer(GovernorLayer {
                config: governor_conf,
            }),
        )
        .fallback_service(ServeDir::new("dist"))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Spawns a long-running background task that checks every 10 seconds if the date has changed.
/// If a new day is detected, it archives team points and resets scores.
pub fn spawn_rollover_task(pool: sqlx::SqlitePool) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            let current_date = Utc::now().format("%Y-%m-%d").to_string();

            if let Ok(row) = sqlx::query("SELECT current_date FROM sys_state WHERE id = 1")
                .fetch_one(&pool)
                .await
            {
                let db_date: String = row.get("current_date");
                if db_date != current_date {
                    if let Ok(winner_row) =
                        sqlx::query("SELECT name FROM teams ORDER BY points DESC LIMIT 1")
                            .fetch_one(&pool)
                            .await
                    {
                        let winner: String = winner_row.get("name");

                        if let Ok(mut tx) = pool.begin().await {
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
}
