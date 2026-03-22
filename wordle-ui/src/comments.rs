/// Returns a tuple of (correct, present, absent) emojis for the given theme.
pub fn get_theme_emojis(theme: &str) -> (&'static str, &'static str, &'static str) {
    match theme {
        "red" => ("🔴", "🔺", "⬛"),
        "orange" => ("🟠", "🔸", "⬛"),
        "yellow" => ("🟡", "✨", "⬛"),
        "green" => ("🟢", "🌱", "⬛"),
        "blue" => ("🔵", "🔹", "⬛"),
        "purple" => ("🟣", "💠", "⬛"),
        _ => ("🟩", "🟨", "⬛"),
    }
}

/// Returns a randomly selected snarky comment based on the game state and chosen team theme.
pub fn get_80s_comment(
    _guess_count: usize,
    is_win: bool,
    is_loss: bool,
    is_hard: bool,
    is_ng: bool,
    theme: &str,
) -> String {
    if is_ng {
        let msgs = match theme {
            "red" => vec![
                "SYSTEM OVERLOADED.",
                "AI IS WINNING.",
                "RESISTANCE IS FUTILE.",
            ],
            "orange" => vec![
                "AI IS NOT VIBING.",
                "IT'S GIVING SYSTEM ERROR.",
                "MAIN CHARACTER DEFEATED.",
            ],
            "yellow" => vec![
                "ERROR 404: LUCK NOT FOUND.",
                "DOES NOT COMPUTE.",
                "BRAIN.EXE HAS STOPPED WORKING.",
            ],
            "green" => vec![
                "GAME OVER, MAN.",
                "INSERT COIN TO CONTINUE.",
                "AI IS SIGMA.",
            ],
            "blue" => vec![
                "SYSTEM DEMANDS RESPECT.",
                "AI IS WORKING OVERTIME.",
                "COMPUTERS WERE BETTER BACK THEN.",
            ],
            "purple" => vec![
                "SYSTEM IS DUSTY.",
                "VACUUM TUBES FAILING.",
                "TELEGRAPH LINE CUT.",
            ],
            _ => vec!["CALCULATING..."],
        };
        return msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string();
    }
    if is_loss {
        let msgs = match theme {
            "red" => vec!["L NOOB 💀", "SKIBIDI TRASH 🚽", "ZERO RIZZ 🥶", "SKILL ISSUE."],
            "orange" => vec![
                "BIG YIKES 😬",
                "NOT THE VIBE 💅",
                "GO TOUCH GRASS 🌿",
                "LITERAL TRASH 🗑️",
            ],
            "yellow" => vec![
                "BETTER LUCK NEXT TIME, GENIUS 🧠",
                "ADULTING IS HARD ☕",
                "I CAN'T EVEN 🤦",
                "TRY GOOGLE NEXT TIME 📱",
            ],
            "green" => vec![
                "TOTALLY BOGUS 🚫",
                "GAG ME WITH A SPOON 🥄",
                "NOT EVEN CLOSE, DUDE 🛹",
                "MAJOR BUMMER 😤",
            ],
            "blue" => vec![
                "BACK TO DRAWING BOARD.",
                "NEEDS MORE ELBOW GREASE.",
                "NOT HACKING IT.",
            ],
            "purple" => vec![
                "LOST IN THE MAIL.",
                "BETTER LUCK NEXT DECADE.",
                "RADIO SILENCE.",
            ],
            _ => vec!["TOTAL BARF BAG."],
        };
        return msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string();
    }
    if is_win {
        let msgs = match theme {
            "red" => vec!["W RIZZ 🥶", "GOAT STATUS 🐐", "NO CAP 🔥", "POGGERS."],
            "orange" => vec![
                "LITERAL CHILLS 🥶",
                "MAIN CHARACTER ENERGY ✨",
                "SLAY QUEEN 💅",
                "VIBE CHECK PASSED ✅",
            ],
            "yellow" => vec![
                "BIG BRAIN ENERGY 🧠",
                "YOU ACTUALLY DID IT 🙃",
                "AVOCADO TOAST FOR YOU 🥑",
                "SIRI, I'M A GENIUS 📱",
            ],
            "green" => vec![
                "TOTALLY TUBULAR 🏄",
                "RADICAL DEPARTURE 🛹",
                "STAY GOLDEN ✨",
                "LIKE, TOTALLY! 🤙",
            ],
            "blue" => vec![
                "BINGO.",
                "ON THE MONEY.",
                "HARD WORK PAYS OFF.",
                "PULLING BOOTSTRAPS.",
            ],
            "purple" => vec![
                "EXTRA RATIONS FOR YOU.",
                "BULLY FOR YOU.",
                "A FINE JOB.",
                "TOP BRASS.",
            ],
            _ => vec!["HACKER!", "GOD MODE."],
        };
        let idx = (js_sys::Math::random() * msgs.len() as f64).floor() as usize;
        let mut msg = msgs[idx].to_string();
        if is_hard {
            msg.push_str(" (HARD MODE ⚡)");
        }
        return msg;
    }

    let msgs = match theme {
        "red" => vec!["BET.", "FR FR.", "SUS.", "LET HIM COOK.", "LOWKEY GOOD."],
        "orange" => vec![
            "GIVING...",
            "IT'S THE GUESS FOR ME.",
            "RENT FREE.",
            "I OOP.",
            "VALID.",
        ],
        "yellow" => vec![
            "IF YOU SAY SO...",
            "SURE, JAN.",
            "STILL AT IT?",
            "LITERALLY THIS.",
        ],
        "green" => vec!["GNARLY.", "WICKED.", "RIGHTEOUS.", "GROOVY.", "CHILL PILL."],
        "blue" => vec!["NEAT.", "SWELL.", "DANDY.", "PEACHY.", "HOKEY."],
        "purple" => vec![
            "SPLENDID.",
            "JOLLY GOOD.",
            "CAPITAL.",
            "SMASHING.",
            "SPIFFING.",
        ],
        _ => vec!["GNARLY.", "WICKED.", "RIGHTEOUS.", "GROOVY."],
    };
    msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_theme_emojis() {
        let (c, p, a) = get_theme_emojis("red");
        assert_eq!(c, "🔴");
        assert_eq!(p, "🔺");
        assert_eq!(a, "⬛");
    }

    #[wasm_bindgen_test]
    fn test_snarky_comment_win() {
        let comment = get_80s_comment(1, true, false, false, false, "red");
        assert!(!comment.is_empty());
    }
}

