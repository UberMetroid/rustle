use leptos::*;
use crate::*;
use wordle_engine::*;

/// Logic for starting New Game Plus.
pub fn get_start_ng_plus(state: AppStateContext) -> impl Fn() {
    move || {
        if !state.daily_game_done.get() {
            let msgs = match state.theme.get().as_str() {
                "red" => vec!["FINISH THE DAILY FIRST BLUD 🛑", "LITERALLY NOT UNLOCKED 🔒", "ONE THING AT A TIME ⏳", "NO CAP BEAT THE DAILY 🧢", "SKIBIDI IMPATIENCE 🚽"],
                "orange" => vec!["ACCESS DENIED BESTIE 🙅", "NOT YET CHAMP 🏆", "WE LOVE AN EAGER QUEEN 👑", "BEAT THE DAILY TO UNLOCK 🔓", "WALK BEFORE YOU RUN 🚶"],
                "yellow" => vec!["FINISH THE DAILY GAME FIRST, GENIUS 🧠", "ADULTING MEANS FINISHING TASKS 📋", "ONE THING AT A TIME ☝️", "EAGER, ARE WE? 👀", "DO THE DAILY FIRST 📅"],
                "green" => vec!["CHILL OUT 🧊", "TAKE A BREATH 🧘", "BEAT THE DAILY FIRST, DUDE ✌️", "NOT YET 🛑", "PATIENCE ⏳"],
                "blue" => vec!["IN MY DAY WE FINISHED CHORES 🧹", "HOLD YOUR HORSES 🐎", "ONE STEP AT A TIME 👣", "BEAT THE DAILY TO UNLOCK 🗝️", "NOT YET ✋"],
                _ => vec!["FINISH THE DAILY GAME FIRST, GENIUS 🧠"],
            };
            state.set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
            set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
            return;
        }
        let was_active = state.is_ng_plus.get();
        state.set_is_ng_plus.set(true);
        state.set_hard_mode.set(true);
        state.set_guesses.set(vec![]);
        state.set_guess_statuses_vec.set(vec![]);
        state.set_game_won.set(false);
        state.set_game_lost.set(false);
        state.set_current_input.set(String::new());
        state.set_session_points.set(0);
        if !was_active {
            let msgs = match state.theme.get().as_str() {
                "red" => vec!["SYSTEM BREACHED 🔓", "NO ESCAPE 🚫", "IT'S OVER FOR YOU 💀"],
                "orange" => vec!["NEW GAME+ ENABLED 💅", "GIVING HARD MODE 😬", "PROTOCOL INITIALIZED 🤖"],
                "yellow" => vec!["NEW GAME+ ENABLED 🎮", "PROTOCOL INITIALIZED ⚙️", "GOOD LUCK 🍀"],
                "green" => vec!["NEW GAME+ 🕹️", "PREPARE YOURSELF 😤", "MAXIMUM EFFORT 💪"],
                "blue" => vec!["NEW GAME+ ENABLED 📺", "PROTOCOL INITIALIZED 📠", "BACK TO WORK 💼"],
                _ => vec!["NEW GAME+ ENABLED 🕹️", "PROTOCOL INITIALIZED ⚙️"],
            };
            state.set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
            set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
        }
        let full_list: Vec<String> = serde_wasm_bindgen::from_value(get_ai_word_list()).unwrap_or_default();
        state.set_ai_pool.set(full_list);
        if let Some(storage) = get_storage() { let _ = storage.remove_item("game-state"); }
    }
}

/// Logic for sharing game results to the clipboard.
pub fn get_share_results(state: AppStateContext) -> impl Fn() {
    move || {
        let is_hard = state.hard_mode.get() || state.is_ng_plus.get();
        let pts = state.session_points.get();
        let t_val = state.theme.get();
        let (correct_e, present_e, absent_e) = get_theme_emojis(&t_val);
        let mut text = format!("RUSTLE {} {}/6 {}{}\n\n", state.solution_data.get().solution_index, if state.game_won.get() { state.guesses.get().len().to_string() } else { "X".to_string() }, if is_hard { "⚡" } else { "" }, if state.is_ng_plus.get() { "+" } else { "" });
        for s_row in state.guess_statuses.get() {
            for s in s_row { text.push_str(match s.as_str() { "correct" => correct_e, "present" => present_e, _ => absent_e }); }
            text.push('\n');
        }
        text.push_str(&format!("\nTEAM {}: {} pts", t_val.to_uppercase(), if pts >= 0 { format!("+{}", pts) } else { pts.to_string() }));
        let _ = web_sys::window().unwrap().navigator().clipboard().write_text(&text);
        state.set_snarky_comment.set("RESULTS COPIED.".to_string());
        set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
    }
}

/// Returns the correct emojis for the current theme.
pub fn get_theme_emojis(theme: &str) -> (&'static str, &'static str, &'static str) {
    match theme {
        "red" => ("🟥", "🟧", "⬛"),
        "orange" => ("🟧", "🟨", "⬛"),
        "yellow" => ("🟩", "🟨", "⬛"),
        "green" => ("🟩", "⬜", "⬛"),
        "blue" => ("🟦", "🟨", "⬛"),
        "purple" => ("🟪", "🟨", "⬛"),
        _ => ("🟩", "🟨", "⬛"),
    }
}
