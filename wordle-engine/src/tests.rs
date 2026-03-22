use super::*;

#[test]
fn test_calculate_status_mask() {
    let mask = calculate_status_mask("apple", "maple");

    let mut expected_mask: u16 = 0;
    // m -> absent -> 0
    // a -> present -> 1
    expected_mask |= 1 << 2;
    // p -> correct -> 2
    expected_mask |= 2 << (2 * 2);
    // l -> correct -> 2
    expected_mask |= 2 << (3 * 2);
    // e -> correct -> 2
    expected_mask |= 2 << (4 * 2);

    assert_eq!(mask, expected_mask);

    // Ensure no overflow!
    let mask_all_correct = calculate_status_mask("words", "words");
    let mut all_correct_mask: u16 = 0;
    for i in 0..5 {
        all_correct_mask |= 2 << (i * 2);
    }
    assert_eq!(mask_all_correct, all_correct_mask);
}

#[test]
fn test_is_word_in_list() {
    // Known correct words
    assert!(is_word_in_list("apple"));
    assert!(is_word_in_list("APPLE"));
    // Random fake words
    assert!(!is_word_in_list("zzzzz"));
    assert!(!is_word_in_list("abcde"));
}

#[test]
fn test_calculate_statuses() {
    // Exact match
    assert_eq!(
        calculate_statuses("apple", "apple"),
        vec!["correct", "correct", "correct", "correct", "correct"]
    );
    // Completely wrong
    assert_eq!(
        calculate_statuses("apple", "ghost"),
        vec!["absent", "absent", "absent", "absent", "absent"]
    );
    // Present and absent mixed
    assert_eq!(
        calculate_statuses("apple", "maple"),
        vec!["absent", "present", "correct", "correct", "correct"]
    );
    // Double letter handling (solution has 2, guess has 1)
    assert_eq!(
        calculate_statuses("apple", "plumb"),
        vec!["present", "present", "absent", "absent", "absent"]
    );
    // Double letter handling (guess has 2, solution has 1)
    assert_eq!(
        calculate_statuses("maple", "apple"),
        vec!["present", "absent", "correct", "correct", "correct"]
    );
}

#[test]
fn test_check_hard_mode_correct_position() {
    let prev_g = vec!["APPLE".to_string()];
    let prev_s = vec![vec![
        "correct".to_string(),
        "absent".to_string(),
        "absent".to_string(),
        "absent".to_string(),
        "absent".to_string(),
    ]];

    // Valid guess, keeps 'A' in the first position
    let err_valid = check_hard_mode_internal("AUDIO", prev_g.clone(), prev_s.clone());
    assert_eq!(err_valid, "");

    // Invalid guess, moves 'A' out of the first position
    let err_invalid = check_hard_mode_internal("GHOST", prev_g.clone(), prev_s.clone());
    assert_eq!(err_invalid, "1ST LETTER MUST BE A.");
}

#[test]
fn test_check_hard_mode_must_contain_present() {
    let prev_g = vec!["APPLE".to_string()];
    let prev_s = vec![vec![
        "present".to_string(),
        "absent".to_string(),
        "absent".to_string(),
        "absent".to_string(),
        "absent".to_string(),
    ]];

    // Valid guess, contains 'A' in a different spot
    let err_valid = check_hard_mode_internal("BREAD", prev_g.clone(), prev_s.clone());
    assert_eq!(err_valid, "");

    // Invalid guess, drops the 'A' entirely
    let err_invalid = check_hard_mode_internal("GHOST", prev_g.clone(), prev_s.clone());
    assert_eq!(err_invalid, "GUESS MUST CONTAIN A.");
}
