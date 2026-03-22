//! Property-based tests for wordle-engine
//!
//! These tests verify game logic properties across random inputs.

use proptest::prelude::*;

/// Verifies that calculate_statuses returns exactly 5 statuses
proptest! {
    #[test]
    fn test_statuses_always_returns_five_elements(
        solution in "[a-z]{5}",
        guess in "[a-z]{5}"
    ) {
        let result = crate::calculate_statuses(&solution, &guess);
        prop_assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_statuses_contains_only_valid_values(
        solution in "[a-z]{5}",
        guess in "[a-z]{5}"
    ) {
        let result = crate::calculate_statuses(&solution, &guess);
        let valid_values = ["correct", "present", "absent"];
        for status in &result {
            prop_assert!(
                valid_values.contains(&status.as_str()),
                "Invalid status: {}",
                status
            );
        }
    }

    #[test]
    fn test_exact_match_returns_all_correct(
        word in "[a-z]{5}"
    ) {
        let result = crate::calculate_statuses(&word, &word);
        prop_assert!(
            result.iter().all(|s| s == "correct"),
            "Exact match should return all correct, got: {:?}",
            result
        );
    }

    #[test]
    fn test_different_short_guess_has_absent_padding(
        solution in "[a-z]{5}",
        guess in "[a-z]{1,4}"
    ) {
        let result = crate::calculate_statuses(&solution, &guess);
        for (i, status) in result.iter().enumerate() {
            if i >= guess.len() {
                prop_assert_eq!(status, "absent", "Position {} should be absent for short guess", i);
            }
        }
    }

    #[test]
    fn test_hard_mode_empty_previous_allows_any_guess(
        guess in "[a-z]{5}"
    ) {
        let result = crate::check_hard_mode_internal(&guess, vec![], vec![]);
        prop_assert!(
            result.is_empty(),
            "Empty previous guesses should allow any guess, got: {}",
            result
        );
    }
}

/// Test word list lookups
mod word_list_tests {
    #[test]
    fn test_is_word_in_list_valid_words() {
        let test_words = ["apple", "crane", "slate", "trace", "arise"];
        for word in test_words {
            assert!(
                crate::is_word_in_list(word),
                "{} should be in word list",
                word
            );
        }
    }

    #[test]
    fn test_is_word_in_list_case_insensitive() {
        assert!(crate::is_word_in_list("APPLE"));
        assert!(crate::is_word_in_list("Apple"));
        assert!(crate::is_word_in_list("ApPlE"));
    }

    #[test]
    fn test_is_word_in_list_invalid_words() {
        let invalid_words = ["zzzzz", "xxxxx", "qqqqq", "abcde", "fghij"];
        for word in invalid_words {
            assert!(
                !crate::is_word_in_list(word),
                "{} should NOT be in word list",
                word
            );
        }
    }
}

/// Test hard mode validation
mod hard_mode_tests {
    #[test]
    fn test_hard_mode_requires_correct_letter_position() {
        let prev_guesses = vec!["APPLE".to_string()];
        let prev_statuses = vec![vec![
            "correct".to_string(),
            "absent".to_string(),
            "absent".to_string(),
            "absent".to_string(),
            "absent".to_string(),
        ]];

        assert!(crate::check_hard_mode_internal(
            "AUDIO",
            prev_guesses.clone(),
            prev_statuses.clone()
        )
        .is_empty());

        let result = crate::check_hard_mode_internal("GHOST", prev_guesses, prev_statuses);
        assert!(result.contains("1ST LETTER MUST BE A"));
    }

    #[test]
    fn test_hard_mode_requires_present_letters() {
        let prev_guesses = vec!["APPLE".to_string()];
        let prev_statuses = vec![vec![
            "present".to_string(),
            "absent".to_string(),
            "absent".to_string(),
            "absent".to_string(),
            "absent".to_string(),
        ]];

        assert!(crate::check_hard_mode_internal(
            "BREAD",
            prev_guesses.clone(),
            prev_statuses.clone()
        )
        .is_empty());

        let result = crate::check_hard_mode_internal("GHOST", prev_guesses, prev_statuses);
        assert!(result.contains("GUESS MUST CONTAIN A"));
    }

    #[test]
    fn test_hard_mode_multiple_requirements() {
        let prev_guesses = vec!["APPLE".to_string(), "CRANE".to_string()];
        let prev_statuses = vec![
            vec![
                "correct".to_string(),
                "absent".to_string(),
                "absent".to_string(),
                "absent".to_string(),
                "absent".to_string(),
            ],
            vec![
                "absent".to_string(),
                "absent".to_string(),
                "present".to_string(),
                "absent".to_string(),
                "absent".to_string(),
            ],
        ];

        let result = crate::check_hard_mode_internal("AXE", prev_guesses, prev_statuses);
        assert!(result.is_empty(), "Should satisfy both requirements");
    }
}
