use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use http_body_util::BodyExt;
use serde_json::json;
use wordle_server::{create_router, db, models::{AppState, GlobalStats}};

async fn setup_app() -> axum::Router {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:?cache=shared")
        .await
        .expect("Failed to connect to test DB");
    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");
    db::seed_db(&pool)
        .await
        .expect("Failed to seed DB");
    let state = AppState { pool };
    create_router(state, ".")
}

fn make_score_request(body: serde_json::Value) -> Request<Body> {
    let connect_info = axum::extract::ConnectInfo(std::net::SocketAddr::from(([127, 0, 0, 1], 8080)));
    Request::builder()
        .method("POST")
        .uri("/api/score")
        .header("Content-Type", "application/json")
        .extension(connect_info)
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

#[tokio::test]
async fn test_get_global_stats_empty() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/global-stats.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let stats: GlobalStats = serde_json::from_slice(&body).expect("Failed to parse stats");
    
    assert_eq!(stats.yesterday_winner, "none");
    assert_eq!(stats.red.points, 0);
    assert_eq!(stats.red.players, 0);
    assert_eq!(stats.blue.points, 0);
}

#[tokio::test]
async fn test_get_global_stats_contains_all_teams() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/global-stats.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let stats: GlobalStats = serde_json::from_slice(&body).expect("Failed to parse stats");
    
    assert!(stats.yellow.points >= 0);
    assert!(stats.red.points >= 0);
    assert!(stats.green.points >= 0);
    assert!(stats.blue.points >= 0);
    assert!(stats.purple.points >= 0);
    assert!(stats.orange.points >= 0);
}

#[tokio::test]
async fn test_submit_score_valid() {
    let app = setup_app().await;

    let payload = json!({
        "player_id": "test-player-1",
        "team": "red",
        "points_delta": 5
    });

    let response = app
        .clone()
        .oneshot(make_score_request(payload))
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let stats_response = app
        .oneshot(
            Request::builder()
                .uri("/global-stats.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    let body = stats_response.into_body().collect().await.unwrap().to_bytes();
    let stats: GlobalStats = serde_json::from_slice(&body).expect("Failed to parse stats");

    assert_eq!(stats.red.points, 5);
    assert_eq!(stats.red.players, 1);
}

#[tokio::test]
async fn test_submit_score_invalid_delta() {
    let app = setup_app().await;

    let payload = json!({
        "player_id": "test-player-2",
        "team": "blue",
        "points_delta": 100
    });

    let response = app
        .oneshot(make_score_request(payload))
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_score_negative_delta() {
    let app = setup_app().await;

    let payload = json!({
        "player_id": "test-player-3",
        "team": "green",
        "points_delta": -3
    });

    let response = app
        .oneshot(make_score_request(payload))
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_submit_score_invalid_team() {
    let app = setup_app().await;

    let payload = json!({
        "player_id": "test-player-4",
        "team": "invalid_team",
        "points_delta": 5
    });

    let response = app
        .oneshot(make_score_request(payload))
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_score_team_switching() {
    let app = setup_app().await;

    let payload1 = json!({
        "player_id": "switching-player",
        "team": "red",
        "points_delta": 10
    });

    app.clone()
        .oneshot(make_score_request(payload1))
        .await
        .expect("Request failed");

    let payload2 = json!({
        "player_id": "switching-player",
        "team": "blue",
        "points_delta": 5
    });

    app.clone()
        .oneshot(make_score_request(payload2))
        .await
        .expect("Request failed");

    let stats_response = app
        .oneshot(
            Request::builder()
                .uri("/global-stats.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    let body = stats_response.into_body().collect().await.unwrap().to_bytes();
    let stats: GlobalStats = serde_json::from_slice(&body).expect("Failed to parse stats");

    assert_eq!(stats.red.players, 0);
    assert_eq!(stats.blue.players, 1);
    assert_eq!(stats.red.points, 10);
    assert_eq!(stats.blue.points, 5);
}

#[tokio::test]
async fn test_submit_score_case_insensitive_team() {
    let app = setup_app().await;

    let payload = json!({
        "player_id": "case-test-player",
        "team": "RED",
        "points_delta": 7
    });

    let response = app
        .oneshot(make_score_request(payload))
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_multiple_players_same_team() {
    let app = setup_app().await;
    
    for i in 0..5 {
        let payload = json!({
            "player_id": format!("player-{}", i),
            "team": "purple",
            "points_delta": 3
        });

        app.clone()
            .oneshot(make_score_request(payload))
            .await
            .expect("Request failed");
    }

    let stats_response = app
        .oneshot(
            Request::builder()
                .uri("/global-stats.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    let body = stats_response.into_body().collect().await.unwrap().to_bytes();
    let stats: GlobalStats = serde_json::from_slice(&body).expect("Failed to parse stats");

    assert_eq!(stats.purple.players, 5);
    assert_eq!(stats.purple.points, 15);
}
