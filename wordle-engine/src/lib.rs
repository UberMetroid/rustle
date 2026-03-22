use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

mod words;
pub use words::{WORDS, VALID_GUESSES};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct SolutionData {
    pub solution: String,
    #[serde(rename = "solutionGameDate")]
    pub solution_game_date: u64,
    #[serde(rename = "solutionIndex")]
    pub solution_index: i64,
    pub tomorrow: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct AdversarialResult {
    pub pattern: Vec<String>,
    pub new_pool: Vec<String>,
}

#[wasm_bindgen]
pub fn get_solution(timestamp: u64) -> JsValue {
    let day = timestamp / 86400000;
    let index = (day % WORDS.len() as u64) as usize;
    let tomorrow = (day + 1) * 86400000;
    
    let sol = SolutionData {
        solution: WORDS[index].to_uppercase(),
        solution_game_date: day,
        solution_index: index as i64,
        tomorrow
    };
    serde_wasm_bindgen::to_value(&sol).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn is_word_in_list(word: &str) -> bool {
    let w = word.to_lowercase();
    WORDS.contains(&w.as_str()) || VALID_GUESSES.contains(&w.as_str())
}

#[wasm_bindgen]
pub fn get_guess_statuses(solution: &str, guess: &str) -> JsValue {
    let statuses = calculate_statuses(solution, guess);
    serde_wasm_bindgen::to_value(&statuses).unwrap_or(JsValue::NULL)
}

pub fn calculate_statuses(solution: &str, guess: &str) -> Vec<String> {
    let sol_chars: Vec<char> = solution.chars().collect();
    let guess_chars: Vec<char> = guess.chars().collect();
    let mut statuses = vec!["absent".to_string(); 5];
    let mut sol_used = [false; 5];
    let mut guess_used = [false; 5];

    for i in 0..5 {
        if i < guess_chars.len() && i < sol_chars.len() && guess_chars[i] == sol_chars[i] {
            statuses[i] = "correct".to_string();
            sol_used[i] = true;
            guess_used[i] = true;
        }
    }

    for i in 0..5 {
        if i < guess_chars.len() && !guess_used[i] {
            for j in 0..5 {
                if !sol_used[j] && guess_chars[i] == sol_chars[j] {
                    statuses[i] = "present".to_string();
                    sol_used[j] = true;
                    break;
                }
            }
        }
    }

    statuses
}

#[wasm_bindgen]
pub fn check_hard_mode(guess: &str, prev_guesses: JsValue, prev_statuses: JsValue) -> String {
    let prev_g: Vec<String> = serde_wasm_bindgen::from_value(prev_guesses).unwrap_or_default();
    let prev_s: Vec<Vec<String>> = serde_wasm_bindgen::from_value(prev_statuses).unwrap_or_default();

    let guess_chars: Vec<char> = guess.chars().collect();

    for (pg, statuses) in prev_g.iter().zip(prev_s.iter()) {
        let prev_chars: Vec<char> = pg.chars().collect();
        
        for i in 0..5 {
            if i < prev_chars.len() && i < statuses.len() && statuses[i] == "correct"
                && (i >= guess_chars.len() || guess_chars[i] != prev_chars[i]) {
                    let nth = match i { 0 => "1ST", 1 => "2ND", 2 => "3RD", 3 => "4TH", _ => "5TH" };
                    return format!("{} LETTER MUST BE {}.", nth, prev_chars[i]);
                }
        }

        let mut required_counts = HashMap::new();
        for i in 0..5 {
            if i < prev_chars.len() && i < statuses.len() && (statuses[i] == "correct" || statuses[i] == "present") {
                *required_counts.entry(prev_chars[i]).or_insert(0) += 1;
            }
        }

        for (c, count) in required_counts {
            let actual = guess_chars.iter().filter(|&&x| x == c).count();
            if actual < count {
                return format!("GUESS MUST CONTAIN {}.", c);
            }
        }
    }
    String::new()
}

#[wasm_bindgen]
pub fn get_ai_word_list() -> JsValue {
    let list: Vec<String> = WORDS.iter().map(|w| w.to_uppercase()).collect();
    serde_wasm_bindgen::to_value(&list).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn get_adversarial_step(guess: &str, current_pool: JsValue) -> JsValue {
    let pool: Vec<String> = serde_wasm_bindgen::from_value(current_pool).unwrap_or_default();
    if pool.is_empty() { return JsValue::NULL; }

    let mut buckets: HashMap<String, Vec<String>> = HashMap::new();

    for sol in &pool {
        let statuses = calculate_statuses(sol, guess);
        let pattern_key = statuses.join(",");
        buckets.entry(pattern_key).or_default().push(sol.clone());
    }

    let mut best_pattern = String::new();
    let mut best_bucket: Vec<String> = Vec::new();

    for (pattern, words) in buckets {
        if words.len() > best_bucket.len() {
            best_pattern = pattern;
            best_bucket = words;
        }
    }

    let result = AdversarialResult {
        pattern: best_pattern.split(',').map(|s| s.to_string()).collect(),
        new_pool: best_bucket
    };
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}
