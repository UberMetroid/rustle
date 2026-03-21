use wasm_bindgen::prelude::*;
use unicode_segmentation::UnicodeSegmentation;
use serde::{Serialize, Deserialize};
use chrono::{Utc, Datelike, Duration, TimeZone};

mod words;
use words::{WORDS, VALID_GUESSES};

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum CharStatus {
    Absent,
    Present,
    Correct,
}

impl CharStatus {
    pub fn to_string(&self) -> String {
        match self {
            CharStatus::Absent => "absent".to_string(),
            CharStatus::Present => "present".to_string(),
            CharStatus::Correct => "correct".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Solution {
    pub solution: String,
    #[serde(rename = "solutionGameDate")]
    pub solution_game_date: u64,
    #[serde(rename = "solutionIndex")]
    pub solution_index: i64,
    pub tomorrow: u64,
}

const FIRST_GAME_DATE_MS: i64 = 1640995200000; // Jan 1 2022 00:00:00 UTC

#[wasm_bindgen]
pub fn get_solution(timestamp_ms: u64) -> JsValue {
    let date = Utc.timestamp_millis_opt(timestamp_ms as i64).unwrap();
    let start_of_day = Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0).unwrap();
    
    let first_game_date = Utc.timestamp_millis_opt(FIRST_GAME_DATE_MS).unwrap();
    
    let diff = start_of_day.signed_duration_since(first_game_date);
    let index = diff.num_days();
    
    let word = WORDS[(index as usize) % WORDS.len()].to_uppercase();
    let tomorrow = (start_of_day + Duration::days(1)).timestamp_millis() as u64;

    let res = Solution {
        solution: word,
        solution_game_date: start_of_day.timestamp_millis() as u64,
        solution_index: index,
        tomorrow,
    };

    serde_wasm_bindgen::to_value(&res).unwrap()
}

#[wasm_bindgen]
pub fn is_word_in_list(word: &str) -> bool {
    let lower_word = word.to_lowercase();
    WORDS.contains(&lower_word.as_str()) || VALID_GUESSES.contains(&lower_word.as_str())
}

#[wasm_bindgen]
pub fn get_guess_statuses(solution: &str, guess: &str) -> JsValue {
    let split_solution: Vec<&str> = solution.graphemes(true).collect();
    let split_guess: Vec<&str> = guess.graphemes(true).collect();

    let mut solution_chars_taken = vec![false; split_solution.len()];
    let mut statuses = vec![CharStatus::Absent; split_guess.len()];
    let mut status_assigned = vec![false; split_guess.len()];

    // Handle all correct cases first
    for (i, &letter) in split_guess.iter().enumerate() {
        if i < split_solution.len() && letter == split_solution[i] {
            statuses[i] = CharStatus::Correct;
            solution_chars_taken[i] = true;
            status_assigned[i] = true;
        }
    }

    // Handle present and absent cases
    for (i, &letter) in split_guess.iter().enumerate() {
        if status_assigned[i] {
            continue;
        }

        let mut found = false;
        for (j, &sol_letter) in split_solution.iter().enumerate() {
            if letter == sol_letter && !solution_chars_taken[j] {
                statuses[i] = CharStatus::Present;
                solution_chars_taken[j] = true;
                found = true;
                break;
            }
        }

        if !found {
            statuses[i] = CharStatus::Absent;
        }
    }

    let status_strings: Vec<String> = statuses.iter().map(|s| s.to_string()).collect();
    serde_wasm_bindgen::to_value(&status_strings).unwrap()
}
