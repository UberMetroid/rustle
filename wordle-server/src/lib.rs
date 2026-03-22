//! Rustle Server Library
//! 
//! This module provides the HTTP server setup, routing, middleware, and background tasks.

pub mod db;
pub mod handlers;
pub mod models;

use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
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
use tracing::{error, info, warn};

use crate::handlers::{get_stats, submit_score};
use crate::models::AppState;

/// Rate limiting configuration
const RATE_LIMIT_PER_MILLIS: u64 = 500;
const RATE_LIMIT_BURST: u32 = 10;

/// Default allowed origins for CORS (comma-separated in env var)
const DEFAULT_ALLOWED_ORIGINS: &str = "*";

/// Middleware to validate API key from `X-API-Key` header
pub async fn api_key_auth(
    request: Request,
    next: Next,
) -> Response {
    let api_key = std::env::var("API_KEY").ok();
    
    if let Some(expected_key) = api_key {
        let provided_key = request
            .headers()
            .get("x-api-key")
            .and_then(|v| v.to_str().ok());
        
        match provided_key {
            Some(key) if key == expected_key => {
                info!(event = "auth_success", "API key authentication successful");
            }
            Some(_) => {
                warn!(event = "auth_failed", reason = "invalid_key", "Invalid API key provided");
                return Response::builder()
                    .status(401)
                    .body(Body::from(r#"{"error":"Invalid API key"}"#))
                    .unwrap();
            }
            None => {
                warn!(event = "auth_failed", reason = "missing_key", "API key header missing");
                return Response::builder()
                    .status(401)
                    .body(Body::from(r#"{"error":"API key required"}"#))
                    .unwrap();
            }
        }
    }
    
    next.run(request).await
}

/// Creates a CORS layer based on ALLOWED_ORIGINS environment variable.
/// Defaults to permissive if not set (for development).
fn create_cors_layer() -> CorsLayer {
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| DEFAULT_ALLOWED_ORIGINS.to_string());
    
    if allowed_origins == "*" {
        info!(event = "cors_config", mode = "permissive", "CORS set to allow any origin");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<&str> = allowed_origins.split(',').map(|s| s.trim()).collect();
        info!(event = "cors_config", mode = "restricted", origins = ?origins);
        
        CorsLayer::new()
            .allow_origin(tower_http::cors::AllowOrigin::list(origins.iter().map(|s| s.parse().unwrap())))
            .allow_methods(Any)
            .allow_headers(Any)
    }
}

/// Creates the Axum router with all routes, state, and middleware configured.
pub fn create_router(state: AppState, dist_path: &str) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(RATE_LIMIT_PER_MILLIS)
            .burst_size(RATE_LIMIT_BURST)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    let cors = create_cors_layer();
    
    Router::new()
        .route("/global-stats.json", get(get_stats))
        .route(
            "/api/score",
            post(submit_score)
                .layer(GovernorLayer {
                    config: governor_conf,
                })
                .layer(axum::middleware::from_fn(api_key_auth)),
        )
        .fallback_service(ServeDir::new(dist_path))
        .layer(CompressionLayer::new())
        .layer(cors)
        .with_state(state)
}

/// Spawns a background task that checks for midnight rollover.
/// Resets scores and archives daily results at UTC midnight.
pub fn spawn_rollover_task(pool: sqlx::SqlitePool) {
    tokio::spawn(async move {
        let mut last_processed_date: Option<String> = None;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            let current_date = Utc::now().format("%Y-%m-%d").to_string();

            if last_processed_date.as_ref() == Some(&current_date) {
                continue;
            }

            // Skip if date hasn't changed in DB yet
            let db_date: Option<String> = sqlx::query(
                "SELECT current_date FROM sys_state WHERE id = 1"
            )
            .fetch_optional(&pool)
            .await
            .inspect_err(|e| error!(event = "db_error", error = %e, "Failed to fetch current date"))
            .ok()
            .flatten()
            .map(|row| row.get::<String, _>("current_date"));

            if db_date.as_ref() == Some(&current_date) {
                last_processed_date = Some(current_date);
                continue;
            }

            // Find winning team
            let winner_result = sqlx::query(
                "SELECT name FROM teams ORDER BY points DESC LIMIT 1"
            )
            .fetch_optional(&pool)
            .await;

            match winner_result {
                Ok(Some(winner_row)) => {
                    let winner: String = winner_row.get("name");
                    info!(
                        event = "daily_rollover",
                        winner = %winner,
                        date = %current_date,
                        "Processing daily rollover"
                    );

                    let mut tx = match pool.begin().await {
                        Ok(tx) => tx,
                        Err(e) => {
                            error!(event = "db_error", error = %e, "Failed to begin transaction");
                            continue;
                        }
                    };

                    // Update system state
                    if let Err(e) = sqlx::query(
                        "UPDATE sys_state SET current_date = ?, yesterday_winner = ? WHERE id = 1"
                    )
                    .bind(&current_date)
                    .bind(&winner)
                    .execute(&mut *tx)
                    .await
                    {
                        error!(event = "db_error", error = %e, "Failed to update sys_state");
                        continue;
                    }

                    // Archive and reset team scores
                    if let Err(e) = sqlx::query(
                        "UPDATE teams SET yesterday_total = points, points = 0, players = 0"
                    )
                    .execute(&mut *tx)
                    .await
                    {
                        error!(event = "db_error", error = %e, "Failed to reset team scores");
                        continue;
                    }

                    // Clear all players for new day
                    if let Err(e) = sqlx::query("DELETE FROM players")
                        .execute(&mut *tx)
                        .await
                    {
                        error!(event = "db_error", error = %e, "Failed to delete players");
                        continue;
                    }

                    match tx.commit().await {
                        Ok(_) => {
                            info!(
                                event = "rollover_complete",
                                winner = %winner,
                                "Daily rollover completed successfully"
                            );
                            last_processed_date = Some(current_date);
                        }
                        Err(e) => {
                            error!(event = "db_error", error = %e, "Failed to commit transaction");
                        }
                    }
                }
                Ok(None) => {
                    warn!(event = "no_winner", "No teams found during rollover");
                }
                Err(e) => {
                    error!(event = "db_error", error = %e, "Failed to find winning team");
                }
            }
        }
    });
}
