use leptos::*;
use crate::*;
use wordle_engine::*;

/// Returns the central keyboard event handler for the game.
pub fn get_on_key(state: AppStateContext) -> impl Fn(String) {
    move |key: String| {
        if state.game_won.get() || state.game_lost.get() {
            if state.daily_game_done.get() {
                if key == "ENTER" { (get_start_ng_plus(state))(); return; }
                if key.len() == 1 && key.chars().next().unwrap_or(' ').is_ascii_alphabetic() { state.set_show_stats.set(true); return; }
                else { return; }
            } else { return; }
        }
        if key == "ENTER" {
            let input = state.current_input.get().to_uppercase();
            let sol = state.solution_data.get().solution.to_uppercase();
            if input.len() < 5 { state.set_jiggle_row.set(true); set_timeout(move || state.set_jiggle_row.set(false), std::time::Duration::from_millis(500)); return; }
            if !is_word_in_list(&input) {
                let msgs = match state.theme.get().as_str() {
                    "red" => vec!["NOT A WORD BLUD 💀", "SKIBIDI SPELLING 🚽", "LITERALLY NOT A WORD 🚫"],
                    "orange" => vec!["NOT A WORD, BESTIE 💅", "YIKES SPELLING 😬", "WE LOVE A MADE UP WORD 🤡"],
                    "yellow" => vec!["DICTIONARY SAYS NO 📚", "I CAN'T EVEN READ THAT 🤦", "SIRI WHAT IS THAT WORD? 📱"],
                    "green" => vec!["BOGUS WORD 🚫", "AS IF ✋", "FAKE NEWS 🗞️"],
                    "blue" => vec!["NOT IN WEBSTER'S 📖", "LEARN TO SPELL 🏫", "BACK TO SCHOOL 🎒"],
                    "purple" => vec!["POPPYCOCK 🎩", "BALDERDASH 🧐", "HOGWASH 📰"],
                    _ => vec!["NOT A WORD."],
                };
                state.set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                state.set_jiggle_row.set(true);
                set_timeout(move || { state.set_snarky_comment.set(String::new()); state.set_jiggle_row.set(false); }, std::time::Duration::from_millis(6000));
                return;
            }

            let (mut new_guesses, mut new_ss_vec) = (state.guesses.get(), state.guess_statuses.get());
            if state.hard_mode.get() || state.is_ng_plus.get() {
                if let (Ok(val_g), Ok(val_s)) = (serde_wasm_bindgen::to_value(&new_guesses), serde_wasm_bindgen::to_value(&new_ss_vec)) {
                    let err = check_hard_mode(&input, val_g, val_s);
                    if !err.is_empty() {
                        let msgs = match state.theme.get().as_str() {
                            "red" => vec!["SKIBIDI ERROR 🚨", "THAT AINT IT 🙅", "AI DETECTS CAP 🧢"],
                            "orange" => vec!["NOT VERY MINDFUL 🛑", "YIKES 😬", "WE LOVE A RULE FOLLOWER 📜"],
                            "yellow" => vec!["READ THE RULES DUMMY 📖", "THAT'S A NO FROM ME 🚫", "SIRI HOW DO I PLAY? 📱"],
                            "green" => vec!["BOGUS GUESS 🚫", "AS IF ✋", "READ THE MANUAL 📖"],
                            "blue" => vec!["FOLLOW THE RULES SONNY 👴", "IN MY DAY WE READ 📰", "RESPECT THE HARD MODE ⚡"],
                            "purple" => vec!["POPPYCOCK 🎩", "BALDERDASH 🧐", "HOGWASH 📰"],
                            _ => vec!["INVALID GUESS."],
                        };
                        state.set_snarky_comment.set(format!("{} {}", msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize], err));
                        state.set_jiggle_row.set(true);
                        set_timeout(move || { state.set_snarky_comment.set(String::new()); state.set_jiggle_row.set(false); }, std::time::Duration::from_millis(6000));
                        return;
                    }
                }
            }

            if state.point_locked_team.get().is_none() { state.set_point_locked_team.set(Some(state.theme.get())); }

            let mut current_pattern = vec![];
            if state.is_ng_plus.get() {
                if let Ok(pool_val) = serde_wasm_bindgen::to_value(&state.ai_pool.get()) {
                    let val = get_adversarial_step(&input, pool_val);
                    if let Ok(res) = serde_wasm_bindgen::from_value::<AdversarialResult>(val) {
                        current_pattern = res.pattern;
                        state.set_ai_pool.set(res.new_pool);
                    }
                }
            } else {
                current_pattern = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &input)).unwrap_or_default();
            }
            if current_pattern.is_empty() { return; }

            let mut turn_pts = 0;
            let current_map = state.char_statuses.get();
            for (i, status) in current_pattern.iter().enumerate() {
                let c = input.chars().nth(i).unwrap_or(' ');
                let existing = current_map.get(&c).map(|s| s.as_str()).unwrap_or("");
                if status == "correct" && existing != "correct" { turn_pts += 2; }
                else if status == "present" && existing != "correct" && existing != "present" { turn_pts += 1; }
            }
            if turn_pts > 0 {
                state.set_session_points.update(|p| *p += turn_pts);
                state.set_win_pulse_trigger.set(format!("+{}", turn_pts));
                set_timeout(move || state.set_win_pulse_trigger.set("".to_string()), std::time::Duration::from_millis(1000));
                post_score(state.theme.get(), turn_pts);
            }

            new_guesses.push(input.clone());
            new_ss_vec.push(current_pattern.clone());
            state.set_guesses.set(new_guesses.clone());
            state.set_guess_statuses_vec.set(new_ss_vec.clone());
            state.set_current_input.set(String::new());
            state.set_is_revealing_row.set(true);
            set_timeout(move || state.set_is_revealing_row.set(false), std::time::Duration::from_millis(2000));

            let is_win = current_pattern.iter().all(|s| s == "correct");
            let is_loss = new_guesses.len() >= 6 && !is_win;
            let mut snark = get_80s_comment(new_guesses.len(), is_win, is_loss, state.hard_mode.get(), state.is_ng_plus.get(), &state.theme.get());
            if is_win {
                state.set_game_won.set(true);
                if !state.is_ng_plus.get() { state.set_daily_game_done.set(true); }
                let final_word = if state.is_ng_plus.get() { state.ai_pool.get().first().cloned().unwrap_or(sol.clone()) } else { sol.clone() };
                let mut bonus = 0;
                if !state.stats.get().scored_words.contains(&final_word) {
                    bonus = if state.is_ng_plus.get() { 5 } else if state.hard_mode.get() { 2 } else { 1 };
                    state.set_session_points.update(|p| *p += bonus);
                    state.set_win_pulse_trigger.set(format!("+{}", bonus));
                    set_timeout(move || state.set_win_pulse_trigger.set("".to_string()), std::time::Duration::from_millis(1500));
                    state.set_stats.update(|s| { s.scored_words.insert(final_word); });
                    post_score(state.theme.get(), bonus);
                }
                if turn_pts + bonus > 0 { snark = format!("{} (+{} PTS)", snark, turn_pts + bonus); }
                set_timeout(move || celebrate(&state.theme.get(), state.hard_mode.get(), state.is_ng_plus.get()), std::time::Duration::from_millis(1800));
                state.set_stats.update(|s| { s.total_games += 1; s.wins += 1; s.current_streak += 1; if s.current_streak > s.best_streak { s.best_streak = s.current_streak; } s.distribution[new_guesses.len() - 1] += 1; });
                set_timeout(move || { state.global_stats_res.refetch(); state.set_show_stats.set(true); }, std::time::Duration::from_millis(3500));
            } else if is_loss {
                state.set_game_lost.set(true);
                if !state.is_ng_plus.get() { state.set_daily_game_done.set(true); }
                state.set_session_points.update(|p| *p -= 1);
                post_score(state.theme.get(), -1);
                state.set_win_pulse_trigger.set("-1".to_string());
                set_timeout(move || state.set_win_pulse_trigger.set("".to_string()), std::time::Duration::from_millis(1500));
                let final_word = if state.is_ng_plus.get() { state.ai_pool.get().first().cloned().unwrap_or(sol.clone()) } else { sol.clone() };
                snark = format!("{} THE WORD WAS {}. (-1 PTS)", snark, final_word);
                state.set_stats.update(|s| { s.total_games += 1; s.current_streak = 0; });
                set_timeout(move || { state.global_stats_res.refetch(); state.set_show_stats.set(true); }, std::time::Duration::from_millis(6000));
            } else if turn_pts > 0 {
                snark = format!("{} (+{} PTS)", snark, turn_pts);
            }
            state.set_snarky_comment.set(snark);
            set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
            if let Some(storage) = get_storage() {
                let s_state = StoredState { guesses: new_guesses, statuses: new_ss_vec, solution: sol, is_ng_plus: state.is_ng_plus.get(), ai_pool_subset: state.ai_pool.get(), daily_done: state.daily_game_done.get(), locked_team: state.point_locked_team.get() };
                if let Ok(s) = serde_json::to_string(&s_state) { let _ = storage.set_item("game-state", &s); }
                if let Ok(s) = serde_json::to_string(&state.stats.get()) { let _ = storage.set_item("game-stats", &s); }
            }
        } else if key == "DELETE" {
            let len = state.current_input.get().len();
            if len > 0 {
                state.set_last_typed_index.set(len as i32 - 1);
                state.set_destroy_trigger.set(js_sys::Date::now().to_string());
                set_timeout(move || { state.set_current_input.update(|s| { s.pop(); }); state.set_last_typed_index.set(-1); state.set_destroy_trigger.set("".to_string()); }, std::time::Duration::from_millis(150));
            }
        } else if state.current_input.get().len() < 5 {
            state.set_last_typed_index.set(state.current_input.get().len() as i32);
            let k = key.to_uppercase();
            state.set_keyboard_pulse.set((k.chars().next().unwrap(), js_sys::Date::now().to_string()));
            state.set_current_input.update(|s| s.push_str(&k));
        }
    }
}
