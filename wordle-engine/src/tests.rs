use super::*;

#[test]
fn test_calculate_status_mask() {
    let mask = calculate_status_mask("apple", "maple");

    let mut expected_mask: u16 = 0;
    expected_mask |= 1 << 2;
    expected_mask |= 2 << (2 * 2);
    expected_mask |= 2 << (3 * 2);
    expected_mask |= 2 << (4 * 2);

    assert_eq!(mask, expected_mask);

    let mask_all_correct = calculate_status_mask("words", "words");
    let mut all_correct_mask: u16 = 0;
    for i in 0..5 {
        all_correct_mask |= 2 << (i * 2);
    }
    assert_eq!(mask_all_correct, all_correct_mask);
}

#[test]
fn test_calculate_status_mask_no_overflow() {
    let mask = calculate_status_mask("zzzzz", "aaaaa");
    assert_eq!(mask, 0);
}

#[test]
fn test_is_word_in_list() {
    assert!(is_word_in_list("apple"));
    assert!(is_word_in_list("APPLE"));
    assert!(is_word_in_list("CrAnE"));
    assert!(!is_word_in_list("zzzzz"));
    assert!(!is_word_in_list("abcde"));
    assert!(!is_word_in_list(""));
}

#[test]
fn test_calculate_statuses_exact_match() {
    assert_eq!(
        calculate_statuses("apple", "apple"),
        vec!["correct", "correct", "correct", "correct", "correct"]
    );
}

#[test]
fn test_calculate_statuses_all_absent() {
    assert_eq!(
        calculate_statuses("apple", "ghost"),
        vec!["absent", "absent", "absent", "absent", "absent"]
    );
}

#[test]
fn test_calculate_statuses_mixed() {
    assert_eq!(
        calculate_statuses("apple", "maple"),
        vec!["absent", "present", "correct", "correct", "correct"]
    );
}

#[test]
fn test_calculate_statuses_double_letter_solution() {
    assert_eq!(
        calculate_statuses("apple", "plumb"),
        vec!["present", "present", "absent", "absent", "absent"]
    );
}

#[test]
fn test_calculate_statuses_double_letter_guess() {
    assert_eq!(
        calculate_statuses("maple", "apple"),
        vec!["present", "absent", "correct", "correct", "correct"]
    );
}

#[test]
fn test_check_hard_mode_no_previous() {
    let result = check_hard_mode_internal("apple", vec![], vec![]);
    assert_eq!(result, "");
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

    let err_valid = check_hard_mode_internal("AUDIO", prev_g.clone(), prev_s.clone());
    assert_eq!(err_valid, "");

    let err_invalid = check_hard_mode_internal("GHOST", prev_g.clone(), prev_s.clone());
    assert_eq!(err_invalid, "1ST LETTER MUST BE A.");
}

#[test]
fn test_check_hard_mode_second_position_valid() {
    let prev_g = vec!["AZZZZ".to_string()];
    let prev_s = vec![vec![
        "correct".to_string(),
        "correct".to_string(),
        "absent".to_string(),
        "absent".to_string(),
        "absent".to_string(),
    ]];

    let err_valid = check_hard_mode_internal("AZURE", prev_g.clone(), prev_s.clone());
    assert_eq!(err_valid, "");

    let err_invalid = check_hard_mode_internal("BAKER", prev_g.clone(), prev_s.clone());
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

    let err_valid = check_hard_mode_internal("BREAD", prev_g.clone(), prev_s.clone());
    assert_eq!(err_valid, "");

    let err_invalid = check_hard_mode_internal("GHOST", prev_g.clone(), prev_s.clone());
    assert_eq!(err_invalid, "GUESS MUST CONTAIN A.");
}

#[test]
fn test_check_hard_mode_multiple_requirements() {
    let prev_g = vec!["APPLE".to_string(), "CRANE".to_string()];
    let prev_s = vec![
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

    let err = check_hard_mode_internal("BRAND", prev_g, prev_s);
    assert!(err.contains("1ST"));
    assert!(err.contains("E"));
}

#[test]
fn test_check_hard_mode_wrong_length() {
    let prev_g = vec!["APPLE".to_string()];
    let prev_s = vec![vec!["correct".to_string(); 5]];

    let result = check_hard_mode_internal("CRAN", prev_g, prev_s);
    assert!(result.contains("1ST LETTER MUST BE"));
}
