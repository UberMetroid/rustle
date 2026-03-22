use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use wordle_server::{create_router, models::{AppState, ScorePayload, GlobalStats}, db};
use tower::ServiceExt; // for `oneshot` and `ready`
use http_body_util::BodyExt; // for `collect`
use serde_json::json;

async fn setup_app() -> axum::Router {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    db::run_migrations(&pool).await;
    db::seed_db(&pool).await;
    let state = AppState { pool };
    create_router(state)
}

#[tokio::test]
async fn test_get_global_stats() {
    let app = setup_app().await;

    let response = app
        .oneshot(Request::builder().uri("/global-stats.json").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let stats: GlobalStats = serde_json::from_slice(&body).unwrap();
    assert_eq!(stats.yesterday_winner, "none");
}

#[tokio::test]
async fn test_submit_score_api() {
    let app = setup_app().await;

    let payload = json!({
        "player_id": "test-api-player",
        "team": "green",
        "points_delta": 5
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/score")
                .header("Content-Type", "application/json")
                .extension(axum::extract::ConnectInfo(
                    std::net::SocketAddr::from(([127, 0, 0, 1], 12345)),
                ))
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
