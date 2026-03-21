use leptos::*;
use wordle_engine::{get_solution, is_word_in_list, get_guess_statuses, get_ai_word_list, get_adversarial_step, SolutionData, AdversarialResult, calculate_statuses};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = cyberBurst)]
    fn confetti();
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct GameStats {
    pub total_games: u32,
    pub wins: u32,
    pub current_streak: u32,
    pub best_streak: u32,
    pub distribution: [u32; 6],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredState {
    pub guesses: Vec<String>,
    pub statuses: Vec<Vec<String>>,
    pub solution: String,
    pub is_ng_plus: bool,
    pub ai_pool: Vec<String>,
}

fn get_storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

fn get_theme_emojis(theme: &str) -> (&str, &str, &str) {
    match theme {
        "dark" => ("⬜", "🟨", "⬛"),
        "red" => ("🟥", "🟨", "⬛"),
        "orange" => ("🟧", "🟨", "⬛"),
        "yellow" => ("🟨", "⬜", "⬛"),
        "green" => ("🟩", "🟨", "⬛"),
        "blue" => ("🟦", "🟨", "⬛"),
        "purple" => ("🟪", "🟨", "⬛"),
        "light" => ("🟩", "🟨", "⬜"),
        _ => ("🟩", "🟨", "⬛"),
    }
}

fn get_80s_comment(tries: usize, is_win: bool, is_loss: bool, is_hard: bool, is_ng: bool) -> String {
    if is_ng {
        if is_win { return "SYSTEM BREACHED. UNREAL.".to_string(); }
        if is_loss { return "THE SYSTEM WINS. LOG OFF.".to_string(); }
        let msgs = vec!["CALCULATING...", "I AM THE SYSTEM.", "GLITCH DETECTED.", "PLAY A GAME?", "ACCESS DENIED.", "DATA CORRUPTION...", "TRON CALLED.", "MAX ENTROPY."];
        return msgs[js_sys::Math::floor(js_sys::Math::random() * msgs.len() as f64) as usize].to_string();
    }
    if is_loss { return "Poseur.".to_string(); }
    if is_win {
        let win_msgs = match tries {
            1 => vec!["HACKER!", "Pure Luck.", "Sus physics.", "God Mode."],
            2 => vec!["Radical!", "Tubular!", "Showoff.", "Excellent!"],
            3 => vec!["Solid mid.", "Typical.", "Choice.", "Right on."],
            4 => vec!["Finally.", "Took your time.", "Getting slow?", "Analog brain."],
            5 => vec!["Panic yet?", "Sweaty.", "Close one.", "Danger Zone."],
            6 => vec!["Barely.", "Yikes.", "Scrub tier.", "Bogus win."],
            _ => vec!["Win."],
        };
        let mut msg = win_msgs[js_sys::Math::floor(js_sys::Math::random() * win_msgs.len() as f64) as usize].to_string();
        if is_hard { msg.push_str(" (Hard Mode)"); }
        return msg;
    }
    let mid_comments = vec!["Gnarly.", "Totally.", "Groovy.", "Neon.", "Retro.", "Bogus?"];
    mid_comments[js_sys::Math::floor(js_sys::Math::random() * mid_comments.len() as f64) as usize].to_string()
}

#[component]
fn Cell(
    value: char, 
    status: String, 
    position: usize, 
    is_revealing: bool, 
    is_completed: bool,
    destroy_trigger: ReadSignal<String>,
    is_last_typed: bool,
    is_hard_mode: bool
) -> impl IntoView {
    let (ring_id, set_ring_id) = create_signal("".to_string());
    let (underline_id, set_underline_id) = create_signal("".to_string());
    let (destroy_id, set_destroy_id) = create_signal("".to_string());
    let (pop_trigger, set_pop_trigger) = create_signal("".to_string());

    create_effect(move |_| {
        if is_last_typed && value != ' ' && !is_completed && !is_revealing {
            set_pop_trigger.set(js_sys::Date::now().to_string());
            if is_hard_mode { set_ring_id.set(js_sys::Date::now().to_string()); }
            else { set_underline_id.set(js_sys::Date::now().to_string()); }
        }
    });

    create_effect(move |_| {
        let trigger = destroy_trigger.get();
        if !trigger.is_empty() && is_last_typed { set_destroy_id.set(trigger); }
    });

    let classes = move || {
        let mut base = "relative w-10 h-10 xs:w-12 xs:h-12 sm:w-14 sm:h-14 border-solid border-2 flex items-center justify-center mx-0.5 text-xl sm:text-4xl font-bold rounded-xl transition-all duration-300".to_string();
        if is_completed || is_revealing {
            if !status.is_empty() { base.push_str(&format!(" {}", status)); }
            else { base.push_str(" cell-neutral"); }
        } else { base.push_str(" cell-neutral"); }
        if is_revealing { base.push_str(" cell-reveal"); }
        else if !pop_trigger.get().is_empty() && !is_completed { base.push_str(" cell-pop"); }
        base
    };
    
    let style = move || if is_revealing { format!("animation-delay: {}ms;", position * 150) } else { "".to_string() };
    
    view! { 
        <div class=classes style=style>
            {move || { let id = ring_id.get(); if !id.is_empty() { view! { <div key=id class="power-ring" /> }.into_view() } else { view! {}.into_view() } }}
            {move || { let id = underline_id.get(); if !id.is_empty() { view! { <div key=id class="power-underline" /> }.into_view() } else { view! {}.into_view() } }}
            {move || { let id = destroy_id.get(); if !id.is_empty() { view! { <div key=id class="destroyed-puff" /> }.into_view() } else { view! {}.into_view() } }}
            <div class="relative z-10">{value.to_uppercase().to_string()}</div>
        </div> 
    }
}

#[component]
fn Row(
    guess: String, 
    statuses: Vec<String>, 
    is_revealing: bool, 
    is_completed: bool, 
    is_jiggling: Signal<bool>,
    destroy_trigger: ReadSignal<String>,
    last_typed_index: i32,
    is_hard_mode: bool
) -> impl IntoView {
    view! {
        <div class=move || format!("flex justify-center mb-1 {}", if is_jiggling.get() { "jiggle" } else { "" })>
            {guess.chars().chain(std::iter::repeat(' ')).take(5).zip(statuses.into_iter().chain(std::iter::repeat("".to_string()))).enumerate().map(|(i, (c, s))| {
                view! { <Cell value=c status=s position=i is_revealing=is_revealing is_completed=is_completed destroy_trigger=destroy_trigger is_last_typed=i as i32 == last_typed_index is_hard_mode=is_hard_mode /> }
            }).collect_view()}
        </div>
    }
}

#[component]
fn Modal(title: String, is_open: ReadSignal<bool>, set_is_open: WriteSignal<bool>, children: ChildrenFn) -> impl IntoView {
    let title_clone = title.clone();
    let children = store_value(children);
    view! {
        <Show when=move || is_open.get()>
            <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black bg-opacity-50" on:click=move |_| set_is_open.set(false)>
                <div class="glass-pad w-full max-w-sm p-6 shadow-2xl transition-all scale-up overflow-y-auto max-h-[90vh]" on:click=move |ev| ev.stop_propagation()>
                    <div class="flex flex-col items-center mb-6 uppercase text-white relative">
                        <div class="absolute right-0 top-0">
                            <button on:click=move |_| set_is_open.set(false) class="text-2xl font-bold hover:text-red-500 transition-colors"> "×" </button>
                        </div>
                        <h2 class="text-2xl font-black tracking-tighter text-center"> {title_clone.clone()} </h2>
                    </div>
                    <div class="text-white text-center"> {children.with_value(|children| children())} </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn App() -> impl IntoView {
    let (guesses, set_guesses) = create_signal(Vec::<String>::new());
    let (guess_statuses, set_guess_statuses_vec) = create_signal(Vec::<Vec<String>>::new());
    let (current_input, set_current_input) = create_signal(String::new());
    let (game_won, set_game_won) = create_signal(false);
    let (game_lost, set_game_lost) = create_signal(false);
    let (show_stats, set_show_stats) = create_signal(false);
    let (show_help, set_show_help) = create_signal(false);
    let (jiggle_row, set_jiggle_row) = create_signal(false);
    let (is_revealing_row, set_is_revealing_row) = create_signal(false);
    let (destroy_trigger, set_destroy_trigger) = create_signal(String::new());
    let (last_typed_index, set_last_typed_index) = create_signal(-1_i32);
    
    let (theme, set_theme) = create_signal("dark".to_string());
    let (hard_mode, set_hard_mode) = create_signal(false);
    let (stats, set_stats) = create_signal(GameStats::default());
    let (keyboard_pulse, set_keyboard_pulse) = create_signal((' ', "".to_string()));
    let (snarky_comment, set_snarky_comment) = create_signal(String::new());

    let (is_ng_plus, set_is_ng_plus) = create_signal(false);
    let (ai_pool, set_ai_pool) = create_signal(Vec::<String>::new());
    let (daily_game_done, set_daily_game_done) = create_signal(false);

    let now = js_sys::Date::now();
    let solution_data = create_memo(move |_| {
        let val = get_solution(now as u64);
        serde_wasm_bindgen::from_value::<SolutionData>(val).unwrap_or_else(|_| {
            SolutionData { solution: "APPLE".to_string(), solution_game_date: 0, solution_index: 0, tomorrow: 0 }
        })
    });

    create_effect(move |_| {
        if let Some(storage) = get_storage() {
            if let Ok(Some(t)) = storage.get_item("color-theme") { set_theme.set(t); }
            if let Ok(Some(h)) = storage.get_item("hard-mode") { set_hard_mode.set(h == "true"); }
            if let Ok(Some(s)) = storage.get_item("game-stats") {
                if let Ok(parsed) = serde_json::from_str::<GameStats>(&s) { set_stats.set(parsed); }
            }
            let sol = solution_data.get().solution;
            if let Ok(Some(saved)) = storage.get_item("game-state") {
                if let Ok(state) = serde_json::from_str::<StoredState>(&saved) {
                    if state.solution == sol {
                        set_guesses.set(state.guesses.clone());
                        set_guess_statuses_vec.set(state.statuses.clone());
                        set_is_ng_plus.set(state.is_ng_plus);
                        set_ai_pool.set(state.ai_pool.clone());
                        if state.guesses.contains(&sol) || (state.is_ng_plus && state.statuses.last().map(|s| s.iter().all(|x| x == "correct")).unwrap_or(false)) {
                            set_game_won.set(true);
                            if !state.is_ng_plus { set_daily_game_done.set(true); }
                        } else if state.guesses.len() >= 6 {
                            set_game_lost.set(true);
                            if !state.is_ng_plus { set_daily_game_done.set(true); }
                        }
                    }
                }
            }
        }
    });

    create_effect(move |_| {
        let t = theme.get();
        if let Some(el) = document().document_element() { let _ = el.set_attribute("class", &format!("theme-{}", t)); }
        if let Some(storage) = get_storage() { let _ = storage.set_item("color-theme", &t); }
    });

    let char_statuses = create_memo(move |_| {
        let mut map = HashMap::new();
        let gs = guesses.get();
        let ss = guess_statuses.get();
        for (g, s_row) in gs.iter().zip(ss.iter()) {
            for (c, s) in g.chars().zip(s_row.iter()) {
                let current = map.entry(c).or_insert(s.clone());
                if s == "correct" { *current = s.clone(); }
                else if s == "present" && *current != "correct" { *current = s.clone(); }
                else if s == "absent" && *current != "correct" && *current != "present" { *current = s.clone(); }
            }
        }
        map
    });

    let start_ng_plus = move || {
        let daily_sol = solution_data.get().solution.to_uppercase();
        let was_active = is_ng_plus.get();
        set_is_ng_plus.set(true);
        set_hard_mode.set(true);
        set_guesses.set(vec![]);
        set_guess_statuses_vec.set(vec![]);
        set_game_won.set(false);
        set_game_lost.set(false);
        set_current_input.set(String::new());
        set_snarky_comment.set(if was_active { "SYSTEM REBOOT." } else { "THE SYSTEM IS ONLINE." }.to_string());
        
        let full_list: Vec<String> = serde_wasm_bindgen::from_value(get_ai_word_list()).unwrap_or_default();
        let mut filtered_pool: Vec<String> = full_list.iter()
            .filter(|w| calculate_statuses(&daily_sol, w).iter().all(|s| s == "absent"))
            .cloned()
            .collect();
        
        if filtered_pool.is_empty() { filtered_pool = full_list; }
        set_ai_pool.set(filtered_pool);
        if let Some(storage) = get_storage() { let _ = storage.remove_item("game-state"); }
    };

    let share_results = move || {
        let is_hard = hard_mode.get() || is_ng_plus.get();
        let comment = snarky_comment.get();
        let shared_comment = if game_lost.get() && !comment.contains("BREACHED") { "TOTAL BUMMER. POSEUR.".to_string() } else { comment };
        
        let current_theme = theme.get();
        let (correct_e, present_e, absent_e) = get_theme_emojis(&current_theme);
        let mut text = format!("RUSTLE {} {}/6 {}{}\n\n", 
            solution_data.get().solution_index, 
            if game_won.get() { guesses.get().len().to_string() } else { "X".to_string() }, 
            if is_hard { "⚡" } else { "" },
            if is_ng_plus.get() { "+" } else { "" }
        );
        let ss = guess_statuses.get();
        for s_row in ss {
            for s in s_row { text.push_str(match s.as_str() { "correct" => correct_e, "present" => present_e, _ => absent_e }); }
            text.push('\n');
        }
        text.push('\n');
        text.push_str(&shared_comment);
        let _ = window().navigator().clipboard().write_text(&text);
        set_snarky_comment.set("Results Copied, poseur.".to_string());
        set_timeout(move || set_snarky_comment.set(String::new()), std::time::Duration::from_millis(2000));
        
        if !is_ng_plus.get() { start_ng_plus(); }
    };

    let on_key = move |key: String| {
        if game_won.get() || game_lost.get() { return; }
        if key == "ENTER" {
            let input = current_input.get().to_uppercase();
            let sol = solution_data.get().solution.to_uppercase();
            if input.len() < 5 {
                set_snarky_comment.set("Way harsh! Needs 5.".to_string());
                set_jiggle_row.set(true);
                set_timeout(move || { set_snarky_comment.set(String::new()); set_jiggle_row.set(false); }, std::time::Duration::from_millis(2000));
                return;
            }
            if !is_word_in_list(&input) {
                set_snarky_comment.set("Not a word, poser.".to_string());
                set_jiggle_row.set(true);
                set_timeout(move || { set_snarky_comment.set(String::new()); set_jiggle_row.set(false); }, std::time::Duration::from_millis(2000));
                return;
            }

            if (hard_mode.get() || is_ng_plus.get()) && !guesses.get().is_empty() {
                let current_guesses = guesses.get();
                let current_ss = guess_statuses.get();
                let mut required_spots: [Option<char>; 5] = [None; 5];
                for (g, ss) in current_guesses.iter().zip(current_ss.iter()) {
                    for (i, (c, s)) in g.chars().zip(ss.iter()).enumerate() {
                        if s == "correct" { required_spots[i] = Some(c); }
                    }
                }
                for (i, &req) in required_spots.iter().enumerate() {
                    if let Some(c) = req {
                        if input.chars().nth(i).unwrap() != c {
                            set_snarky_comment.set(format!("{} goes at spot {}", c, i + 1));
                            set_jiggle_row.set(true);
                            set_timeout(move || { set_snarky_comment.set(String::new()); set_jiggle_row.set(false); }, std::time::Duration::from_millis(2000));
                            return;
                        }
                    }
                }
                let mut required_letters: HashMap<char, usize> = HashMap::new();
                for (g, ss) in current_guesses.iter().zip(current_ss.iter()) {
                    let mut current_g_counts: HashMap<char, usize> = HashMap::new();
                    for (c, s) in g.chars().zip(ss.iter()) {
                        if s == "correct" || s == "present" { *current_g_counts.entry(c).or_insert(0) += 1; }
                    }
                    for (c, count) in current_g_counts {
                        let entry = required_letters.entry(c).or_insert(0);
                        if count > *entry { *entry = count; }
                    }
                }
                for (c, &req_count) in &required_letters {
                    let input_count = input.chars().filter(|&ic| ic == *c).count();
                    if input_count < req_count {
                        set_snarky_comment.set(format!("Needs more {}. Bogus.", c));
                        set_jiggle_row.set(true);
                        set_timeout(move || { set_snarky_comment.set(String::new()); set_jiggle_row.set(false); }, std::time::Duration::from_millis(2000));
                        return;
                    }
                }
                let statuses_map = char_statuses.get();
                for c in input.chars() {
                    if let Some(status) = statuses_map.get(&c) {
                        if status == "absent" {
                            set_snarky_comment.set(format!("{} is trash. Drop it.", c));
                            set_jiggle_row.set(true);
                            set_timeout(move || { set_snarky_comment.set(String::new()); set_jiggle_row.set(false); }, std::time::Duration::from_millis(2000));
                            return;
                        }
                    }
                }
            }

            let mut new_guesses = guesses.get();
            let mut new_ss_vec = guess_statuses.get();
            new_guesses.push(input.clone());

            let mut current_pattern = vec![];
            if is_ng_plus.get() {
                let pool_val = serde_wasm_bindgen::to_value(&ai_pool.get()).unwrap();
                let val = get_adversarial_step(&input, pool_val);
                if let Ok(res) = serde_wasm_bindgen::from_value::<AdversarialResult>(val) {
                    current_pattern = res.pattern;
                    set_ai_pool.set(res.new_pool.clone());
                }
            } else {
                current_pattern = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &input)).unwrap_or_default();
            }
            
            if current_pattern.is_empty() { return; }

            new_ss_vec.push(current_pattern.clone());
            set_guesses.set(new_guesses.clone());
            set_guess_statuses_vec.set(new_ss_vec.clone());
            set_current_input.set(String::new());

            if let Some(storage) = get_storage() {
                let state = StoredState { 
                    guesses: new_guesses.clone(), statuses: new_ss_vec.clone(), solution: sol.clone(), is_ng_plus: is_ng_plus.get(), ai_pool: ai_pool.get()
                };
                let _ = storage.set_item("game-state", &serde_json::to_string(&state).unwrap());
            }

            set_is_revealing_row.set(true);
            set_timeout(move || set_is_revealing_row.set(false), std::time::Duration::from_millis(2000));
            
            let is_win = current_pattern.iter().all(|s| s == "correct");
            let is_loss = new_guesses.len() >= 6 && !is_win;
            set_snarky_comment.set(get_80s_comment(new_guesses.len(), is_win, is_loss, hard_mode.get(), is_ng_plus.get()));

            if is_win {
                set_game_won.set(true);
                if !is_ng_plus.get() { set_daily_game_done.set(true); }
                
                if is_ng_plus.get() {
                    set_timeout(move || confetti(), std::time::Duration::from_millis(1800));
                    set_timeout(move || confetti(), std::time::Duration::from_millis(2000));
                    set_timeout(move || confetti(), std::time::Duration::from_millis(2200));
                    set_timeout(move || confetti(), std::time::Duration::from_millis(2400));
                } else {
                    set_timeout(move || confetti(), std::time::Duration::from_millis(1800));
                }

                set_stats.update(|s| {
                    s.total_games += 1; s.wins += 1; s.current_streak += 1;
                    if s.current_streak > s.best_streak { s.best_streak = s.current_streak; }
                    s.distribution[new_guesses.len() - 1] += 1;
                });
                if let Some(storage) = get_storage() { let _ = storage.set_item("game-stats", &serde_json::to_string(&stats.get()).unwrap()); }
                set_timeout(move || set_show_stats.set(true), std::time::Duration::from_millis(3500));
            } else if is_loss {
                set_game_lost.set(true);
                if !is_ng_plus.get() { set_daily_game_done.set(true); }
                let final_word = if is_ng_plus.get() { ai_pool.get().first().cloned().unwrap_or(sol) } else { sol };
                set_snarky_comment.set(format!("SCRUB TIER. IT WAS {}.", final_word));
                set_stats.update(|s| { s.total_games += 1; s.current_streak = 0; });
                if let Some(storage) = get_storage() { let _ = storage.set_item("game-stats", &serde_json::to_string(&stats.get()).unwrap()); }
                set_timeout(move || set_show_stats.set(true), std::time::Duration::from_millis(3500));
            }
        } else if key == "DELETE" {
            let len = current_input.get().len();
            if len > 0 {
                set_last_typed_index.set(len as i32 - 1);
                set_destroy_trigger.set(js_sys::Date::now().to_string());
                set_timeout(move || {
                    set_current_input.update(|s| { s.pop(); });
                    set_last_typed_index.set(-1);
                    set_destroy_trigger.set("".to_string());
                }, std::time::Duration::from_millis(150));
            }
        } else if current_input.get().len() < 5 {
            let next_idx = current_input.get().len() as i32;
            set_last_typed_index.set(next_idx);
            let k = key.to_uppercase();
            let c = k.chars().next().unwrap();
            set_keyboard_pulse.set((c, js_sys::Date::now().to_string()));
            set_current_input.update(|s| s.push_str(&k));
        }
    };

    let _ = window_event_listener(leptos::ev::keydown, move |ev| {
        let key = ev.key();
        if show_stats.get() {
            if key == "Enter" && (game_won.get() || game_lost.get()) {
                share_results();
                set_show_stats.set(false);
            }
            return;
        }
        if show_help.get() { return; }
        
        if key == "Enter" { on_key("ENTER".to_string()); }
        else if key == "Backspace" { on_key("DELETE".to_string()); }
        else if key.len() == 1 {
            let c = key.chars().next().unwrap();
            if c.is_ascii_alphabetic() { on_key(c.to_uppercase().to_string()); }
        }
    });

    view! {
        <div class="flex flex-col h-full transition-all duration-500 bg-app-bg text-app-text overflow-hidden">
            <header class="w-full flex flex-col items-center py-4 shrink-0">
                <h1 class="text-3xl sm:text-5xl font-black tracking-tighter italic text-center title-text uppercase">"RUSTLE"</h1>
                <div class="min-h-8 flex items-center justify-center w-full max-w-sm px-4 mt-1">
                    {move || {
                        let snark = snarky_comment.get();
                        if !snark.is_empty() {
                            let color = if game_won.get() { "text-green-400" } else if game_lost.get() { "text-red-400" } else { "text-theme-primary" };
                            view! { <div key=snark.clone() class=format!("text-xs sm:text-sm font-black uppercase tracking-widest {} snarky-toast text-center", color)> {snark} </div> }.into_view()
                        } else { view! {}.into_view() }
                    }}
                </div>
            </header>

            <main class="flex-1 flex items-center justify-center w-full max-w-4xl mx-auto px-2 sm:px-4 min-h-0">
                <aside class="flex flex-col gap-4 py-4 shrink-0">
                    <button on:click=move |_| set_show_stats.set(true) title="Score" class="correct-pad w-10 h-10 sm:w-14 sm:h-14 flex items-center justify-center rounded-2xl shadow-lg border-2 border-transparent transition-all active:scale-90">
                        <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path></svg>
                    </button>
                    <button on:click=move |_| set_show_help.set(true) title="How to Play" class="correct-pad w-10 h-10 sm:w-14 sm:h-14 flex items-center justify-center rounded-2xl shadow-lg border-2 border-transparent transition-all active:scale-90">
                        <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                    </button>
                    <button 
                        on:click=move |_| if guesses.get().is_empty() { set_hard_mode.update(|h| *h = !*h) } 
                        title="Hard Mode" 
                        class=move || format!("w-10 h-10 sm:w-14 sm:h-14 flex items-center justify-center rounded-2xl shadow-lg border-2 transition-all active:scale-90 {}", if hard_mode.get() || is_ng_plus.get() { "correct-pad border-transparent" } else { "cell-neutral border-current" })
                    >
                        <svg class=move || format!("w-6 h-6 sm:w-8 sm:h-8 transition-all {}", if hard_mode.get() || is_ng_plus.get() { "text-yellow-300 scale-110 drop-shadow-[0_0_12px_rgba(253,224,71,1)]" } else { "text-current opacity-40" }) fill="currentColor" viewBox="0 0 24 24">
                            <path d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                        </svg>
                    </button>
                    <button 
                        on:click=move |_| start_ng_plus()
                        disabled=move || !daily_game_done.get()
                        title="New Game+" 
                        class=move || format!("w-10 h-10 sm:w-14 sm:h-14 flex items-center justify-center rounded-2xl shadow-lg border-2 transition-all active:scale-90 {}", if is_ng_plus.get() { "ai-active-pad border-transparent shadow-[0_0_25px_rgba(255,0,255,1)]" } else if daily_game_done.get() { "cell-neutral border-current" } else { "opacity-30 grayscale cursor-not-allowed border-current" })
                    >
                        <span class=move || format!("text-2xl sm:text-4xl font-black transition-all {}", if is_ng_plus.get() { "text-white scale-110" } else { "text-current opacity-40" })>"+"</span>
                    </button>
                </aside>

                <div class="flex-1 flex flex-col items-center justify-center min-h-0 py-4 px-2 sm:px-8">
                    <div class="flex flex-col gap-1 sm:gap-2 h-full max-h-[500px] aspect-[5/6]">
                        {move || {
                            let gs = guesses.get();
                            let ss = guess_statuses.get();
                            let is_rev = is_revealing_row.get();
                            let len = gs.len();
                            let hard = hard_mode.get() || is_ng_plus.get();
                            gs.into_iter().zip(ss.into_iter()).enumerate().map(move |(i, (g, s))| { 
                                view! { <Row guess=g.to_uppercase() statuses=s is_revealing=is_rev && i == len-1 is_completed=true is_jiggling=Signal::derive(|| false) destroy_trigger=destroy_trigger last_typed_index=-1 is_hard_mode=hard /> } 
                            }).collect_view()
                        }}
                        {move || if guesses.get().len() < 6 && !game_won.get() { 
                            let current_input = current_input.get().to_uppercase();
                            let last_idx = last_typed_index.get();
                            let hard = hard_mode.get() || is_ng_plus.get();
                            view! { <Row guess=current_input statuses=vec![] is_revealing=false is_completed=false is_jiggling=Signal::derive(move || jiggle_row.get()) destroy_trigger=destroy_trigger last_typed_index=last_idx is_hard_mode=hard /> }.into_view() 
                        } else { view! {}.into_view() }}
                        {move || {
                            let hard = hard_mode.get() || is_ng_plus.get();
                            (0..(6_usize.saturating_sub(guesses.get().len() + if guesses.get().len() < 6 && !game_won.get() { 1 } else { 0 }))).map(move |_| { view! { <Row guess="".to_string() statuses=vec![] is_revealing=false is_completed=false is_jiggling=Signal::derive(|| false) destroy_trigger=destroy_trigger last_typed_index=-1 is_hard_mode=hard /> } }).collect_view()
                        }}
                    </div>
                </div>

                <aside class="flex flex-col items-center justify-center py-4 shrink-0">
                    <div class="glass-pad p-3 rounded-3xl flex items-center shadow-lg relative z-[50]">
                        {move || {
                            let themes = vec!["dark", "red", "orange", "yellow", "green", "blue", "purple", "light"];
                            let current = theme.get();
                            let index = themes.iter().position(|&t| t == current).unwrap_or(0);
                            view! { 
                                <div class="theme-slider-container">
                                    <input type="range" min="0" max="7" step="1" value=index class="theme-slider" on:input=move |ev| { 
                                        let val = event_target_value(&ev).parse::<usize>().unwrap_or(0); 
                                        set_theme.set(themes[val].to_string()); 
                                    } /> 
                                </div>
                            }
                        }}
                    </div>
                </aside>
            </main>

            <footer class="w-full max-w-2xl mx-auto p-2 pb-6 shrink-0">
                {move || {
                    let rows = vec![vec!['Q','W','E','R','T','Y','U','I','O','P'], vec!['A','S','D','F','G','H','J','K','L'], vec!['Z','X','C','V','B','N','M']];
                    rows.into_iter().enumerate().map(|(i, row)| {
                        view! {
                            <div class="flex justify-center mb-1.5 w-full gap-1 sm:gap-1.5 px-1">
                                {if i == 2 { view! { <button class="key-large px-3 rounded-xl font-black transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg flex-[1.5] key-neutral flex items-center justify-center" on:click=move |_| on_key("ENTER".to_string())> <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path></svg> </button> }.into_view() } else { view! {}.into_view() }}
                                {row.into_iter().map(|c| {
                                    let status = move || char_statuses.get().get(&c).cloned().unwrap_or_default();
                                    let status_class = move || match status().as_str() { "correct" => "correct", "present" => "present", "absent" => "absent", _ => "key-neutral" };
                                    let pulse = move || if keyboard_pulse.get().0 == c { keyboard_pulse.get().1 } else { "".to_string() };
                                    let hard = hard_mode.get() || is_ng_plus.get();
                                    view! { 
                                        <button class=move || format!("key-large relative rounded-xl font-black flex-1 min-w-[30px] transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg border-2 border-transparent {}", status_class()) on:click=move |_| on_key(c.to_string())> 
                                            {move || { 
                                                let id = pulse(); 
                                                if !id.is_empty() { 
                                                    if hard { view! { <div key=id class="power-ring" /> }.into_view() }
                                                    else { view! { <div key=id class="power-underline" /> }.into_view() }
                                                } else { view! {}.into_view() } 
                                            }}
                                            {c.to_string()} 
                                        </button> 
                                    }
                                }).collect_view()}
                                {if i == 2 { view! { <button class="key-large px-3 rounded-xl font-black transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg flex-[1.5] key-neutral flex items-center justify-center" on:click=move |_| on_key("DELETE".to_string())> <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2M3 12l6.414 6.414a2 2 0 001.414.586H19a2 2 0 002-2V7a2 2 0 00-2-2h-8.172a2 2 0 00-1.414.586L3 12z"></path></svg> </button> }.into_view() } else { view! {}.into_view() }}
                            </div>
                        }
                    }).collect_view()
                }}
            </footer>

            <Modal title="How to Play".to_string() is_open=show_help set_is_open=set_show_help>
                <div class="flex flex-col gap-6 text-white text-center">
                    <div class="space-y-4">
                        <div class="space-y-3">
                            <div class="flex flex-col items-center gap-1">
                                <div class="flex">
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent correct font-black text-white">"R"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"U"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"T"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                                </div>
                                <div class="text-[10px] opacity-70 text-center">"R is in the word and in the correct spot."</div>
                            </div>
                            <div class="flex flex-col items-center gap-1">
                                <div class="flex">
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral font-bold">"W"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent present mx-0.5 font-black text-black">"O"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"R"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"D"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                                </div>
                                <div class="text-[10px] opacity-70 text-center">"O is in the word but in the wrong spot."</div>
                            </div>
                            <div class="flex flex-col items-center gap-1">
                                <div class="flex">
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral font-bold">"V"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"A"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"G"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent absent mx-0.5 font-black text-white">"U"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"E"</div>
                                </div>
                                <div class="text-[10px] opacity-70 text-center">"U is not in the word in any spot."</div>
                            </div>
                        </div>
                    </div>
                </div>
            </Modal>

            <Modal title="Statistics".to_string() is_open=show_stats set_is_open=set_show_stats>
                <div class="flex flex-col items-center text-center text-white">
                    <div class="grid grid-cols-4 w-full gap-4 mb-8">
                        <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().total_games}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Played"</div></div>
                        <div class="flex flex-col"><div class="text-3xl font-black">{move || if stats.get().total_games > 0 { stats.get().wins * 100 / stats.get().total_games } else { 0 }}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Win %"</div></div>
                        <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().current_streak}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Streak"</div></div>
                        <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().best_streak}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Best"</div></div>
                    </div>
                    
                    <h3 class="text-xs font-bold uppercase mb-4 tracking-widest text-theme-primary text-center">"Guess Distribution"</h3>
                    <div class="w-full space-y-1.5 mb-8 text-left">
                        {move || stats.get().distribution.iter().enumerate().map(|(i, count)| {
                            let wins = stats.get().wins;
                            let width = if wins > 0 { (*count as f32 * 100.0 / wins as f32).max(10.0) } else { 10.0 };
                            view! { <div class="flex items-center gap-2 text-xs text-white"><div class="w-2">{i+1}</div><div class="bg-gray-600 text-white font-black p-0.5 rounded text-right pr-2 transition-all duration-1000" style=format!("width: {}%", width)>{*count}</div></div> }
                        }).collect_view()}
                    </div>

                    <div class="w-full border-t border-white border-opacity-10 pt-6 mb-6 text-center">
                        <h3 class="text-xs font-bold uppercase mb-4 tracking-widest text-theme-primary">"Global Statistics"</h3>
                        <div class="grid grid-cols-3 w-full gap-2">
                            <div class="flex flex-col p-2 rounded-lg bg-white bg-opacity-5"><div class="text-xl font-black">"1.2M"</div><div class="text-[7px] uppercase opacity-50">"Cracked"</div></div>
                            <div class="flex flex-col p-2 rounded-lg bg-white bg-opacity-5"><div class="text-xl font-black">"4.2"</div><div class="text-[7px] uppercase opacity-50">"Avg Ops"</div></div>
                            <div class="flex flex-col p-2 rounded-lg bg-white bg-opacity-5"><div class="text-xl font-black">"88%"</div><div class="text-[7px] uppercase opacity-50">"Efficiency"</div></div>
                        </div>
                    </div>

                    <Show when=move || game_won.get() || game_lost.get()>
                        <button on:click=move |_| share_results() class="w-full bg-green-500 hover:bg-green-600 text-white font-black py-3 rounded-xl shadow-lg flex items-center justify-center gap-2 transition-all active:scale-95 uppercase tracking-widest"> "SHARE" </button>
                    </Show>
                </div>
            </Modal>
        </div>
    }
}

fn main() { 
    console_error_panic_hook::set_once(); 
    let root = document().get_element_by_id("root").expect("could not find #root element");
    mount_to(root.unchecked_into(), || view! { <App/> }) 
}
