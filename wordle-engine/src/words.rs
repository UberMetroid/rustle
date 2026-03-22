use once_cell::sync::Lazy;

/// A statically loaded list of the daily solution words.
pub static WORDS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    include_str!("words.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .collect()
});

/// A statically loaded list of additional valid guess words.
pub static VALID_GUESSES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    include_str!("valid_guesses.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .collect()
});
