use wasm_bindgen::prelude::*;
use std::collections::HashMap;

mod words;
use words::WORDS;

#[wasm_bindgen]
pub fn get_solution(timestamp: u64) -> JsValue {
    let day = timestamp / 86400000;
    let index = (day % WORDS.len() as u64) as usize;
    let tomorrow = (day + 1) * 86400000;
    
    let sol = serde_json::json!({
        "solution": WORDS[index].to_uppercase(),
        "solutionGameDate": day,
        "solutionIndex": index as i64,
        "tomorrow": tomorrow
    });
    serde_wasm_bindgen::to_value(&sol).unwrap()
}

#[wasm_bindgen]
pub fn is_word_in_list(word: &str) -> bool {
    let w = word.to_lowercase();
    WORDS.contains(&w.as_str())
}

#[wasm_bindgen]
pub fn get_guess_statuses(solution: &str, guess: &str) -> JsValue {
    let statuses = calculate_statuses(solution, guess);
    serde_wasm_bindgen::to_value(&statuses).unwrap()
}

// ADVERSARIAL AI LOGIC (Absurdle)
#[wasm_bindgen]
pub fn get_ai_word_list() -> JsValue {
    let list: Vec<String> = WORDS.iter().map(|w| w.to_uppercase()).collect();
    serde_wasm_bindgen::to_value(&list).unwrap()
}

#[wasm_bindgen]
pub fn get_adversarial_step(guess: &str, current_pool: JsValue) -> JsValue {
    let pool: Vec<String> = serde_wasm_bindgen::from_value(current_pool).unwrap_or_default();
    if pool.is_empty() { return JsValue::NULL; }

    // Map: Pattern String -> List of words that produce that pattern
    let mut buckets: HashMap<String, Vec<String>> = HashMap::new();

    for sol in &pool {
        let statuses = calculate_statuses(sol, guess);
        let pattern_key = statuses.join(",");
        buckets.entry(pattern_key).or_insert_with(Vec::new).push(sol.clone());
    }

    // Find the bucket with the most words (Adversarial choice)
    let mut best_pattern = String::new();
    let mut best_bucket: Vec<String> = Vec::new();

    for (pattern, words) in buckets {
        if words.len() > best_bucket.len() {
            best_pattern = pattern;
            best_bucket = words;
        }
    }

    let result = serde_json::json!({
        "pattern": best_pattern.split(',').collect::<Vec<&str>>(),
        "newPool": best_bucket
    });
    serde_wasm_bindgen::to_value(&result).unwrap()
}

fn calculate_statuses(solution: &str, guess: &str) -> Vec<String> {
    let sol_chars: Vec<char> = solution.chars().collect();
    let guess_chars: Vec<char> = guess.chars().collect();
    let mut statuses = vec!["absent".to_string(); 5];
    let mut sol_used = vec![false; 5];
    let mut guess_used = vec![false; 5];

    // First pass: Find correct spots (Green)
    for i in 0..5 {
        if guess_chars[i] == sol_chars[i] {
            statuses[i] = "correct".to_string();
            sol_used[i] = true;
            guess_used[i] = true;
        }
    }

    // Second pass: Find present letters (Yellow)
    for i in 0..5 {
        if !guess_used[i] {
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
