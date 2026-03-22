pub mod db;
pub mod handlers;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};
use chrono::Utc;
use sqlx::Row;
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::handlers::{get_stats, submit_score};
use crate::models::AppState;

const RATE_LIMIT_PER_MILLIS: u64 = 500;
const RATE_LIMIT_BURST: u32 = 10;

fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

pub fn create_router(state: AppState, dist_path: &str) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(RATE_LIMIT_PER_MILLIS)
            .burst_size(RATE_LIMIT_BURST)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    Router::new()
        .route("/global-stats.json", get(get_stats))
        .route(
            "/api/score",
            post(submit_score).layer(GovernorLayer {
                config: governor_conf,
            }),
        )
        .fallback_service(ServeDir::new(dist_path))
        .layer(CompressionLayer::new())
        .layer(create_cors_layer())
        .with_state(state)
}

pub fn spawn_rollover_task(pool: sqlx::SqlitePool) {
    tokio::spawn(async move {
        let mut last_processed_date: Option<String> = None;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            let current_date = Utc::now().format("%Y-%m-%d").to_string();

            if last_processed_date.as_ref() == Some(&current_date) {
                continue;
            }

            let result: Option<(String, String)> = sqlx::query(
                "SELECT current_date, yesterday_winner FROM sys_state WHERE id = 1"
            )
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten()
            .map(|row| {
                let db_date: String = row.get("current_date");
                let winner: String = row.get("yesterday_winner");
                (db_date, winner)
            });

            if let Some((db_date, _)) = result {
                if db_date == current_date {
                    last_processed_date = Some(current_date);
                    continue;
                }
            }

            if let Ok(winner_row) = sqlx::query(
                "SELECT name FROM teams ORDER BY points DESC LIMIT 1"
            )
            .fetch_one(&pool)
            .await
            {
                let winner: String = winner_row.get("name");

                if let Ok(mut tx) = pool.begin().await {
                    let _ = sqlx::query(
                        "UPDATE sys_state SET current_date = ?, yesterday_winner = ? WHERE id = 1"
                    )
                    .bind(&current_date)
                    .bind(&winner)
                    .execute(&mut *tx)
                    .await;

                    let _ = sqlx::query(
                        "UPDATE teams SET yesterday_total = points, points = 0, players = 0"
                    )
                    .execute(&mut *tx)
                    .await;

                    let _ = sqlx::query("DELETE FROM players").execute(&mut *tx).await;

                    let _ = tx.commit().await;
                    last_processed_date = Some(current_date);
                }
            }
        }
    });
}
