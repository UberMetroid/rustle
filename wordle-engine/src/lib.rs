use wasm_bindgen::prelude::*;
use unicode_segmentation::UnicodeSegmentation;
use serde::{Serialize, Deserialize};

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

    // Convert to vector of strings for compatibility with existing TS code if needed,
    // or use serde to pass the enum array.
    let status_strings: Vec<String> = statuses.iter().map(|s| s.to_string()).collect();
    serde_wasm_bindgen::to_value(&status_strings).unwrap()
}
