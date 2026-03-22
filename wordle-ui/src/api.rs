use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;
use web_sys::window;

use crate::*;

/// Represents the statistics for a single team.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct TeamData {
    /// Total points for the team today.
    pub points: i32,
    /// Current number of players on this team.
    pub players: u32,
    /// Team's total points from yesterday.
    pub yesterday_total: i32,
}

/// The global state data fetched from the server.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct GlobalStats {
    pub yellow: TeamData,
    pub red: TeamData,
    pub green: TeamData,
    pub blue: TeamData,
    pub purple: TeamData,
    pub orange: TeamData,
    /// The team name that won yesterday.
    pub yesterday_winner: String,
    /// Current server time.
    pub server_utc_timestamp: u64,
}

/// User's local game statistics.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct GameStats {
    pub total_games: u32,
    pub wins: u32,
    pub current_streak: u32,
    pub best_streak: u32,
    /// Win distribution by number of guesses (1-6).
    pub distribution: [u32; 6],
    /// Set of words already scored to prevent double bonus.
    pub scored_words: HashSet<String>,
}

/// The state of a current game session, saved to local storage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredState {
    pub guesses: Vec<String>,
    pub statuses: Vec<Vec<String>>,
    pub solution: String,
    pub is_ng_plus: bool,
    pub ai_pool_subset: Vec<String>,
    pub daily_done: bool,
    pub locked_team: Option<String>,
}

/// Helper to access browser local storage.
pub fn get_storage() -> Option<web_sys::Storage> {
    web_sys::window().unwrap().local_storage().ok().flatten()
}

/// Fetches the latest global statistics from the backend.
pub async fn fetch_global_stats() -> GlobalStats {
    let window = window();
    let url = "/global-stats.json";
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    if let Ok(request) = web_sys::Request::new_with_str_and_init(url, &opts) {
        if let Ok(resp_value) =
            wasm_bindgen_futures::JsFuture::from(window.unwrap().fetch_with_request(&request)).await
        {
            if let Ok(resp) = resp_value.dyn_into::<web_sys::Response>() {
                if resp.status() == 200 {
                    if let Ok(json_promise) = resp.json() {
                        if let Ok(json_value) =
                            wasm_bindgen_futures::JsFuture::from(json_promise).await
                        {
                            if let Ok(stats) =
                                serde_wasm_bindgen::from_value::<GlobalStats>(json_value)
                            {
                                return stats;
                            }
                        }
                    }
                }
            }
        }
    }
    GlobalStats::default()
}

/// Submits a point delta for a specific team to the backend.
pub fn post_score(team: String, points_delta: i32) {
    let mut opts = web_sys::RequestInit::new();
    opts.set_method("POST");

    let payload = ScorePayload {
        player_id: get_player_id(),
        team,
        points_delta,
    };

    if let Ok(body) = serde_json::to_string(&payload) {
        opts.set_body(&JsValue::from_str(&body));

        let headers = web_sys::Headers::new().unwrap();
        let _ = headers.append("Content-Type", "application/json");
        opts.set_headers(&headers);

        let window = window();
        if let Ok(request) = web_sys::Request::new_with_str_and_init("/api/score", &opts) {
            let _ = window.unwrap().fetch_with_request(&request);
        }
    }
}

/// Retrieves or generates a unique player ID stored in the browser.
pub fn get_player_id() -> String {
    if let Some(storage) = get_storage() {
        if let Ok(Some(id)) = storage.get_item("player-id") {
            return id;
        }
        let new_id = format!("p-{}", js_sys::Math::random());
        let _ = storage.set_item("player-id", &new_id);
        return new_id;
    }
    "anonymous".to_string()
}

/// The internal payload sent to the API.
#[derive(Serialize)]
pub struct ScorePayload {
    pub player_id: String,
    pub team: String,
    pub points_delta: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_game_stats_default() {
        let stats = GameStats::default();
        assert_eq!(stats.total_games, 0);
        assert_eq!(stats.distribution, [0; 6]);
    }
}
