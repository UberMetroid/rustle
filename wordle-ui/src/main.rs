use leptos::*;
use wordle_engine::{get_solution, is_word_in_list, check_hard_mode, get_guess_statuses, get_ai_word_list, get_adversarial_step, SolutionData, AdversarialResult};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = cyberpunkVictory)]
    fn celebrate(theme: &str, is_hard: bool, is_ng_plus: bool);
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
struct TeamData {
    pub points: i32,
    pub players: u32,
    pub yesterday_total: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
struct GlobalStats {
    pub yellow: TeamData,
    pub red: TeamData,
    pub green: TeamData,
    pub blue: TeamData,
    pub orange: TeamData,
    pub yesterday_winner: String,
    pub server_utc_timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
struct GameStats {
    pub total_games: u32,
    pub wins: u32,
    pub current_streak: u32,
    pub best_streak: u32,
    pub distribution: [u32; 6],
    pub scored_words: HashSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredState {
    pub guesses: Vec<String>,
    pub statuses: Vec<Vec<String>>,
    pub solution: String,
    pub is_ng_plus: bool,
    pub ai_pool_subset: Vec<String>,
    pub daily_done: bool,
    pub locked_team: Option<String>,
}

fn get_storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

async fn fetch_global_stats() -> GlobalStats {
    let window = window();
    let url = "/global-stats.json";
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    if let Ok(request) = web_sys::Request::new_with_str_and_init(url, &opts) {
        if let Ok(resp_value) = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await {
            if let Ok(resp) = resp_value.dyn_into::<web_sys::Response>() {
                if resp.status() == 200 {
                    if let Ok(json_promise) = resp.json() {
                        if let Ok(json_value) = wasm_bindgen_futures::JsFuture::from(json_promise).await {
                            if let Ok(stats) = serde_wasm_bindgen::from_value::<GlobalStats>(json_value) {
                                return stats;
                            }
                        }
                    }
                }
            }
        }
    }
    GlobalStats {
        yellow: TeamData { points: 0, players: 0, yesterday_total: 0 },
        red: TeamData { points: 0, players: 0, yesterday_total: 0 },
        green: TeamData { points: 0, players: 0, yesterday_total: 0 },
        blue: TeamData { points: 0, players: 0, yesterday_total: 0 },
        orange: TeamData { points: 0, players: 0, yesterday_total: 0 },
        yesterday_winner: "none".to_string(),
        server_utc_timestamp: js_sys::Date::now() as u64,
    }
}

fn get_player_id() -> String {
    if let Some(storage) = get_storage() {
        if let Ok(Some(id)) = storage.get_item("player-id") {
            return id;
        }
        let new_id = js_sys::Math::random().to_string();
        let _ = storage.set_item("player-id", &new_id);
        return new_id;
    }
    js_sys::Math::random().to_string()
}

#[derive(Serialize)]
struct ScorePayload {
    player_id: String,
    team: String,
    points_delta: i32,
}

fn post_score(team: String, points_delta: i32) {
    let payload = ScorePayload { player_id: get_player_id(), team, points_delta };
    if let Ok(json) = serde_json::to_string(&payload) {
        let opts = web_sys::RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&JsValue::from_str(&json));

        if let Ok(headers) = web_sys::Headers::new() {
            let _ = headers.set("Content-Type", "application/json");
            opts.set_headers(&headers);
        }

        let window = window();
        if let Ok(request) = web_sys::Request::new_with_str_and_init("/api/score", &opts) {
            let _ = window.fetch_with_request(&request);
        }
    }
}

fn get_theme_emojis(theme: &str) -> (&str, &str, &str) {
    match theme {
        "yellow" => ("🟨", "🟧", "⬛"),
        "red" => ("🟥", "🟨", "⬛"),
        "green" => ("🟩", "🟨", "⬛"),
        "blue" => ("🟦", "🟨", "⬛"),
        "orange" => ("🟧", "🟨", "⬛"),
        _ => ("🟩", "🟨", "⬛"),
    }
}

fn get_80s_comment(tries: usize, is_win: bool, is_loss: bool, is_hard: bool, is_ng: bool, theme: &str) -> String {
    if is_ng {
        if is_win { return "SYSTEM BREACHED.".to_string(); }
        if is_loss { return "SYSTEM WINS.".to_string(); }
        let msgs = match theme {
            "red" => vec!["SKIBIDI TOILET ENERGY.", "SYSTEM AINT CAPPING.", "AI IS SIGMA."],
            "orange" => vec!["AI IS NOT VIBING.", "IT'S GIVING SYSTEM ERROR.", "AI TOUCHED GRASS."],
            "yellow" => vec!["SYSTEM IS STRESSED.", "AI NEEDS A COFFEE.", "ADULTING FAILED."],
            "green" => vec!["WHATEVER, MEATBAG.", "SYSTEM IS BORED.", "AI SAYS TALK TO THE HAND."],
            "blue" => vec!["SYSTEM DEMANDS RESPECT.", "AI IS WORKING OVERTIME.", "COMPUTERS WERE BETTER BACK THEN."],
            _ => vec!["CALCULATING..."]
        };
        return msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string();
    }
    if is_loss { 
        let msgs = match theme {
            "red" => vec!["L + RATIO.", "MASSIVE SKILL ISSUE.", "YOU FELL OFF."],
            "orange" => vec!["NOT VERY MINDFUL.", "BIG YIKES.", "WE LOVE A TRYING MOMENT."],
            "yellow" => vec!["BIG OOF.", "I CAN'T EVEN.", "SHOULD HAVE GOOGLED IT."],
            "green" => vec!["TOTAL BARF BAG.", "GAG ME WITH A SPOON.", "YOU GOT OWNED."],
            "blue" => vec!["BACK TO DRAWING BOARD.", "NEEDS MORE ELBOW GREASE.", "NOT HACKING IT."],
            _ => vec!["TOTAL BARF BAG."]
        };
        return msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string();
    }
    if is_win {
        let msgs = match theme {
            "red" => vec!["W RIZZ.", "ABSOLUTE CINEMA.", "GOAT LEVEL.", "NO PRINTS."],
            "orange" => vec!["ATE AND LEFT NO CRUMBS.", "PERIODT.", "SLAYED THAT.", "MAIN CHARACTER."],
            "yellow" => vec!["YAAAAS.", "I DID A THING.", "HARRY POTTER MAGIC.", "SPARKJOY ACHIEVED."],
            "green" => vec!["RADICAL.", "BODACIOUS.", "TOTALLY TUBULAR.", "EXCELLENT."],
            "blue" => vec!["BINGO.", "ON THE MONEY.", "HARD WORK PAYS OFF.", "PULLING BOOTSTRAPS."],
            _ => vec!["HACKER!", "GOD MODE."]
        };
        let idx = (js_sys::Math::random() * msgs.len() as f64).floor() as usize;
        let mut msg = msgs[idx].to_string();
        if is_hard { msg.push_str(&format!(" ({} TRIES)", tries)); }
        return msg;
    }
    
    let msgs = match theme {
        "red" => vec!["BET.", "FR FR.", "SUS.", "LET HIM COOK.", "LOWKEY GOOD."],
        "orange" => vec!["GIVING...", "IT'S THE GUESS FOR ME.", "RENT FREE.", "I OOP.", "VALID."],
        "yellow" => vec!["DOGGO APPROVES.", "SMOL BRAIN MOMENT.", "CHEEKY.", "SPICY TAKE.", "LITERALLY THIS."],
        "green" => vec!["GNARLY.", "WICKED.", "RIGHTEOUS.", "GROOVY.", "CHILL PILL."],
        "blue" => vec!["NEAT.", "SWELL.", "DANDY.", "PEACHY.", "HOKEY."],
        _ => vec!["GNARLY.", "WICKED.", "RIGHTEOUS.", "GROOVY."]
    };
    msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string()
}

#[component]
fn Cell(value: char, status: String, position: usize, is_revealing: bool, is_completed: bool, destroy_trigger: ReadSignal<String>, is_last_typed: bool, is_hard_mode: bool) -> impl IntoView {
    let (ring_id, set_ring_id) = create_signal("".to_string());
    let (underline_id, set_underline_id) = create_signal("".to_string());
    let (destroy_id, set_destroy_id) = create_signal("".to_string());
    let (pop_trigger, set_pop_trigger) = create_signal("".to_string());
    create_effect(move |_| { if is_last_typed && value != ' ' && !is_completed && !is_revealing { set_pop_trigger.set(js_sys::Date::now().to_string()); if is_hard_mode { set_ring_id.set(js_sys::Date::now().to_string()); } else { set_underline_id.set(js_sys::Date::now().to_string()); } } });
    create_effect(move |_| { let trigger = destroy_trigger.get(); if !trigger.is_empty() && is_last_typed { set_destroy_id.set(trigger); } });
    let classes = move || {
        let mut base = "relative w-10 h-10 xs:w-12 xs:h-12 sm:w-14 sm:h-14 border-solid border-2 flex items-center justify-center mx-0.5 text-xl sm:text-4xl font-bold rounded-xl transition-all duration-300".to_string();
        if is_completed || is_revealing { if !status.is_empty() { base.push_str(&format!(" {}", status)); } else { base.push_str(" cell-neutral"); } } else { base.push_str(" cell-neutral"); }
        if is_revealing { base.push_str(" cell-reveal"); } else if !pop_trigger.get().is_empty() && !is_completed { base.push_str(" cell-pop"); }
        base
    };
    let style = move || if is_revealing { format!("animation-delay: {}ms;", position * 150) } else { "".to_string() };
    view! { <div class=classes style=style> {move || { let id = ring_id.get(); if !id.is_empty() { view! { <div key=id class="power-ring" /> }.into_view() } else { view! {}.into_view() } }} {move || { let id = underline_id.get(); if !id.is_empty() { view! { <div key=id class="power-underline" /> }.into_view() } else { view! {}.into_view() } }} {move || { let id = destroy_id.get(); if !id.is_empty() { view! { <div key=id class="destroyed-puff" /> }.into_view() } else { view! {}.into_view() } }} <div class="relative z-10">{value.to_uppercase().to_string()}</div> </div> }
}

#[component]
fn Row(guess: String, statuses: Vec<String>, is_revealing: bool, is_completed: bool, is_jiggling: Signal<bool>, destroy_trigger: ReadSignal<String>, last_typed_index: i32, is_hard_mode: bool) -> impl IntoView {
    view! { <div class=move || format!("flex justify-center mb-1 {}", if is_jiggling.get() { "jiggle" } else { "" })> {guess.chars().chain(std::iter::repeat(' ')).take(5).zip(statuses.into_iter().chain(std::iter::repeat("".to_string()))).enumerate().map(|(i, (c, s))| { view! { <Cell value=c status=s position=i is_revealing=is_revealing is_completed=is_completed destroy_trigger=destroy_trigger is_last_typed=i as i32 == last_typed_index is_hard_mode=is_hard_mode /> } }).collect_view()} </div> }
}

#[component]
fn Modal(title: String, is_open: ReadSignal<bool>, set_is_open: WriteSignal<bool>, children: ChildrenFn) -> impl IntoView {
    let title_clone = title.clone();
    let children = store_value(children);
    view! { <Show when=move || is_open.get()> <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black bg-opacity-50" on:click=move |_| set_is_open.set(false)> <div class="glass-pad w-full max-w-sm p-6 shadow-2xl transition-all scale-up overflow-y-auto max-h-[90vh]" on:click=move |ev| ev.stop_propagation()> <div class="flex flex-col items-center mb-6 uppercase text-white relative"> <div class="absolute right-0 top-0"> <button on:click=move |_| set_is_open.set(false) class="text-2xl font-bold hover:text-red-500 transition-colors"> "×" </button> </div> <h2 class="text-2xl font-black tracking-tighter text-center"> {title_clone.clone()} </h2> </div> <div class="text-white text-center"> {children.with_value(|children| children())} </div> </div> </div> </Show> }
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
    let (theme, set_theme) = create_signal("yellow".to_string());
    let (hard_mode, set_hard_mode) = create_signal(false);
    let (stats, set_stats) = create_signal(GameStats::default());
    let (keyboard_pulse, set_keyboard_pulse) = create_signal((' ', "".to_string()));
    let (snarky_comment, set_snarky_comment) = create_signal(String::new());
    let (is_ng_plus, set_is_ng_plus) = create_signal(false);
    let (ai_pool, set_ai_pool) = create_signal(Vec::<String>::new());
    let (daily_game_done, set_daily_game_done) = create_signal(false);
    let (win_pulse_trigger, set_win_pulse_trigger) = create_signal(String::new());
    let (session_points, set_session_points) = create_signal(0_i32);
    let (point_locked_team, set_point_locked_team) = create_signal(None::<String>);

    let global_stats_res = create_local_resource(move || (), |_| fetch_global_stats());
    let solution_data = create_memo(move |_| {
        let ts = match global_stats_res.get() {
            Some(stats) => stats.server_utc_timestamp,
            None => js_sys::Date::now() as u64,
        };
        let val = get_solution(ts);
        serde_wasm_bindgen::from_value::<SolutionData>(val).unwrap_or_else(|_| { SolutionData { solution: "APPLE".to_string(), solution_game_date: 0, solution_index: 0, tomorrow: 0 } }) 
    });

    create_effect(move |_| {
        if let Some(storage) = get_storage() {
            if let Ok(Some(t)) = storage.get_item("color-theme") { set_theme.set(t); }
            if let Ok(Some(h)) = storage.get_item("hard-mode") { set_hard_mode.set(h == "true"); }
            if let Ok(Some(s)) = storage.get_item("game-stats") { if let Ok(parsed) = serde_json::from_str::<GameStats>(&s) { set_stats.set(parsed); } }
            let sol = solution_data.get().solution;
            if let Ok(Some(saved)) = storage.get_item("game-state") {
                if let Ok(state) = serde_json::from_str::<StoredState>(&saved) {
                    if state.solution == sol {
                        set_guesses.set(state.guesses.clone()); set_guess_statuses_vec.set(state.statuses.clone()); set_is_ng_plus.set(state.is_ng_plus); set_daily_game_done.set(state.daily_done); set_point_locked_team.set(state.locked_team);
                        if state.guesses.contains(&sol) || (state.is_ng_plus && state.statuses.last().map(|s| s.iter().all(|x| x == "correct")).unwrap_or(false)) { set_game_won.set(true); }
                        else if state.guesses.len() >= 6 { set_game_lost.set(true); }
                        if state.is_ng_plus { let full_list: Vec<String> = serde_wasm_bindgen::from_value(get_ai_word_list()).unwrap_or_default(); set_ai_pool.set(full_list); }
                    }
                }
            }
        }
    });

    create_effect(move |_| {
        let t = theme.get();
        if let Some(el) = document().document_element() {
            let _ = el.set_attribute("class", &format!("theme-{}", t));
        }
        if let Some(storage) = get_storage() {
            let _ = storage.set_item("color-theme", &t);
        }
    });

    let char_statuses = create_memo(move |_| {
        let mut map = HashMap::new(); let gs = guesses.get(); let ss = guess_statuses.get();
        for (g, s_row) in gs.iter().zip(ss.iter()) {
            for (c, s) in g.chars().zip(s_row.iter()) {
                let current = map.entry(c).or_insert_with(|| s.clone());
                if s == "correct" || (s == "present" && *current != "correct") || (s == "absent" && *current != "correct" && *current != "present") {
                    *current = s.clone();
                }
            }
        }
        map
    });

    let start_ng_plus = move || {
        if !daily_game_done.get() {
            let msgs = match theme.get().as_str() {
                "red" => vec!["FINISH THE DAILY FIRST BLUD.", "LITERALLY NOT UNLOCKED.", "ONE THING AT A TIME.", "NO CAP BEAT THE DAILY.", "SKIBIDI IMPATIENCE."],
                "orange" => vec!["ACCESS DENIED BESTIE.", "NOT YET CHAMP.", "WE LOVE AN EAGER QUEEN.", "BEAT THE DAILY TO UNLOCK.", "WALK BEFORE YOU RUN."],
                "yellow" => vec!["FINISH THE DAILY GAME FIRST, GENIUS.", "ADULTING MEANS FINISHING TASKS.", "ONE THING AT A TIME.", "EAGER, ARE WE?", "DO THE DAILY FIRST."],
                "green" => vec!["CHILL OUT.", "TAKE A BREATH.", "BEAT THE DAILY FIRST, DUDE.", "NOT YET.", "PATIENCE."],
                "blue" => vec!["IN MY DAY WE FINISHED CHORES.", "HOLD YOUR HORSES.", "ONE STEP AT A TIME.", "BEAT THE DAILY TO UNLOCK.", "NOT YET."],
                _ => vec!["FINISH THE DAILY GAME FIRST, GENIUS."]
            };
            set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
            set_timeout(move || set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
            return;
        }
        let was_active = is_ng_plus.get();
        set_is_ng_plus.set(true); set_hard_mode.set(true); set_guesses.set(vec![]); set_guess_statuses_vec.set(vec![]); set_game_won.set(false); set_game_lost.set(false); set_current_input.set(String::new()); set_session_points.set(0);
        if !was_active { 
            let msgs = match theme.get().as_str() {
                "red" => vec!["SYSTEM BREACHED.", "NO ESCAPE.", "IT'S OVER FOR YOU."],
                "orange" => vec!["NEW GAME+ ENABLED.", "GIVING HARD MODE.", "PROTOCOL INITIALIZED."],
                "yellow" => vec!["NEW GAME+ ENABLED.", "PROTOCOL INITIALIZED.", "GOOD LUCK."],
                "green" => vec!["NEW GAME+.", "PREPARE YOURSELF.", "MAXIMUM EFFORT."],
                "blue" => vec!["NEW GAME+ ENABLED.", "PROTOCOL INITIALIZED.", "BACK TO WORK."],
                _ => vec!["NEW GAME+ ENABLED.", "PROTOCOL INITIALIZED."]
            };
            set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string()); 
        }
        let full_list: Vec<String> = serde_wasm_bindgen::from_value(get_ai_word_list()).unwrap_or_default();
        set_ai_pool.set(full_list); if let Some(storage) = get_storage() { let _ = storage.remove_item("game-state"); }
    };

    let share_results = move || {
        let is_hard = hard_mode.get() || is_ng_plus.get();
        let pts = session_points.get();
        let t_val = theme.get();
        let (correct_e, present_e, absent_e) = get_theme_emojis(&t_val);
        let mut text = format!("RUSTLE {} {}/6 {}{}\n\n", solution_data.get().solution_index, if game_won.get() { guesses.get().len().to_string() } else { "X".to_string() }, if is_hard { "⚡" } else { "" }, if is_ng_plus.get() { "+" } else { "" });
        for s_row in guess_statuses.get() { for s in s_row { text.push_str(match s.as_str() { "correct" => correct_e, "present" => present_e, _ => absent_e }); } text.push('\n'); }
        text.push_str(&format!("\nTEAM {}: {} pts", t_val.to_uppercase(), if pts >= 0 { format!("+{}", pts) } else { pts.to_string() }));
        let _ = window().navigator().clipboard().write_text(&text);
        
        set_show_stats.set(false);
        if !is_ng_plus.get() && daily_game_done.get() {
            start_ng_plus();
            set_snarky_comment.set("RESULTS COPIED. NOW BEAT THE SYSTEM.".to_string());
        } else {
            set_snarky_comment.set("RESULTS COPIED.".to_string());
        }
        set_timeout(move || set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
    };

    let on_key = move |key: String| {
        if game_won.get() || game_lost.get() {
            if daily_game_done.get() {
                if key == "ENTER" { start_ng_plus(); return; }
                if key.len() == 1 && key.chars().next().unwrap().is_ascii_alphabetic() {
                    start_ng_plus();
                } else { return; }
            } else { return; }
        }
        if key == "ENTER" {
            let input = current_input.get().to_uppercase();
            let sol = solution_data.get().solution.to_uppercase();
            if input.len() < 5 { set_jiggle_row.set(true); set_timeout(move || set_jiggle_row.set(false), std::time::Duration::from_millis(500)); return; }
            if !is_word_in_list(&input) { 
                let msgs = match theme.get().as_str() {
                    "red" => vec!["NOT A WORD BLUD.", "SKIBIDI SPELLING.", "LITERALLY NOT A WORD."],
                    "orange" => vec!["NOT A WORD, BESTIE.", "YIKES SPELLING.", "WE LOVE A MADE UP WORD."],
                    "yellow" => vec!["DICTIONARY SAYS NO.", "I CAN'T EVEN READ THAT.", "SIRI WHAT IS THAT WORD?"],
                    "green" => vec!["BOGUS WORD.", "AS IF.", "FAKE NEWS."],
                    "blue" => vec!["NOT IN WEBSTER'S.", "LEARN TO SPELL.", "BACK TO SCHOOL."],
                    _ => vec!["NOT A WORD."]
                };
                set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                set_jiggle_row.set(true); 
                set_timeout(move || { set_snarky_comment.set(String::new()); set_jiggle_row.set(false); }, std::time::Duration::from_millis(6000)); 
                return; 
            }

            let mut new_guesses = guesses.get(); let mut new_ss_vec = guess_statuses.get();
            if hard_mode.get() || is_ng_plus.get() {
                let err = check_hard_mode(&input, serde_wasm_bindgen::to_value(&new_guesses).unwrap(), serde_wasm_bindgen::to_value(&new_ss_vec).unwrap());
                if !err.is_empty() {
                    let msgs = match theme.get().as_str() {
                        "red" => vec!["SKIBIDI ERROR.", "THAT AINT IT.", "AI DETECTS CAP."],
                        "orange" => vec!["NOT VERY MINDFUL.", "YIKES.", "WE LOVE A RULE FOLLOWER."],
                        "yellow" => vec!["READ THE RULES DUMMY.", "THAT'S A NO FROM ME.", "SIRI HOW DO I PLAY?"],
                        "green" => vec!["BOGUS GUESS.", "AS IF.", "READ THE MANUAL."],
                        "blue" => vec!["FOLLOW THE RULES SONNY.", "IN MY DAY WE READ.", "RESPECT THE HARD MODE."],
                        _ => vec!["INVALID GUESS."]
                    };
                    let prefix_str = msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string();
                    set_snarky_comment.set(format!("{} {}", prefix_str, err)); set_jiggle_row.set(true); set_timeout(move || { set_snarky_comment.set(String::new()); set_jiggle_row.set(false); }, std::time::Duration::from_millis(6000));
                    return;
                }
            }

            if point_locked_team.get().is_none() { set_point_locked_team.set(Some(theme.get())); }

            let mut current_pattern = vec![];
            if is_ng_plus.get() { let pool_val = serde_wasm_bindgen::to_value(&ai_pool.get()).unwrap(); let val = get_adversarial_step(&input, pool_val); if let Ok(res) = serde_wasm_bindgen::from_value::<AdversarialResult>(val) { current_pattern = res.pattern; set_ai_pool.set(res.new_pool.clone()); } }
            else { current_pattern = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &input)).unwrap_or_default(); }
            if current_pattern.is_empty() { return; }
            
            let mut turn_pts = 0;
            let current_map = char_statuses.get();
            for (i, status) in current_pattern.iter().enumerate() {
                let c = input.chars().nth(i).unwrap();
                let existing = current_map.get(&c).map(|s| s.as_str()).unwrap_or("");
                if status == "correct" && existing != "correct" { turn_pts += 2; }
                else if status == "present" && existing != "correct" && existing != "present" { turn_pts += 1; }
            }
            if turn_pts > 0 { set_session_points.update(|p| *p += turn_pts); set_win_pulse_trigger.set(format!("+{}", turn_pts)); set_timeout(move || set_win_pulse_trigger.set("".to_string()), std::time::Duration::from_millis(1000)); post_score(theme.get(), turn_pts); }

            new_guesses.push(input.clone()); new_ss_vec.push(current_pattern.clone());
            set_guesses.set(new_guesses.clone()); set_guess_statuses_vec.set(new_ss_vec.clone()); set_current_input.set(String::new());
            set_is_revealing_row.set(true); set_timeout(move || set_is_revealing_row.set(false), std::time::Duration::from_millis(2000));
            
            let is_win = current_pattern.iter().all(|s| s == "correct");
            let is_loss = new_guesses.len() >= 6 && !is_win;
            let mut snark = get_80s_comment(new_guesses.len(), is_win, is_loss, hard_mode.get(), is_ng_plus.get(), &theme.get());
            if is_win {
                set_game_won.set(true); if !is_ng_plus.get() { set_daily_game_done.set(true); }
                let final_word = if is_ng_plus.get() { ai_pool.get().first().cloned().unwrap_or(sol.clone()) } else { sol.clone() };
                let mut bonus = 0;
                if !stats.get().scored_words.contains(&final_word) {
                    bonus = if is_ng_plus.get() { 5 } else if hard_mode.get() { 2 } else { 1 };
                    set_session_points.update(|p| *p += bonus);
                    set_win_pulse_trigger.set(format!("+{}", bonus));
                    set_timeout(move || set_win_pulse_trigger.set("".to_string()), std::time::Duration::from_millis(1500));
                    set_stats.update(|s| { s.scored_words.insert(final_word); });
                    post_score(theme.get(), bonus);
                }
                if turn_pts + bonus > 0 { snark = format!("{} (+{} PTS)", snark, turn_pts + bonus); }
                set_timeout(move || celebrate(&theme.get(), hard_mode.get(), is_ng_plus.get()), std::time::Duration::from_millis(1800));
                set_stats.update(|s| { s.total_games += 1; s.wins += 1; s.current_streak += 1; if s.current_streak > s.best_streak { s.best_streak = s.current_streak; } s.distribution[new_guesses.len() - 1] += 1; });
                set_timeout(move || { global_stats_res.refetch(); set_show_stats.set(true); }, std::time::Duration::from_millis(3500));
            } else if is_loss {
                set_game_lost.set(true); if !is_ng_plus.get() { set_daily_game_done.set(true); }
                set_session_points.update(|p| *p -= 1);
                post_score(theme.get(), -1);
                set_win_pulse_trigger.set("-1".to_string());
                set_timeout(move || set_win_pulse_trigger.set("".to_string()), std::time::Duration::from_millis(1500));
                let final_word = if is_ng_plus.get() { ai_pool.get().first().cloned().unwrap_or(sol.clone()) } else { sol.clone() };
                snark = format!("{} THE WORD WAS {}. (-1 PTS)", snark, final_word);
                set_stats.update(|s| { s.total_games += 1; s.current_streak = 0; });
                set_timeout(move || { global_stats_res.refetch(); set_show_stats.set(true); }, std::time::Duration::from_millis(6000));
            } else {
                if turn_pts > 0 { snark = format!("{} (+{} PTS)", snark, turn_pts); }
            }
            set_snarky_comment.set(snark);
            if let Some(storage) = get_storage() {
                let state = StoredState { guesses: new_guesses, statuses: new_ss_vec, solution: sol, is_ng_plus: is_ng_plus.get(), ai_pool_subset: ai_pool.get().clone(), daily_done: daily_game_done.get(), locked_team: point_locked_team.get() };
                if let Ok(state_str) = serde_json::to_string(&state) {
                    let _ = storage.set_item("game-state", &state_str);
                }
                if let Ok(stats_str) = serde_json::to_string(&stats.get()) {
                    let _ = storage.set_item("game-stats", &stats_str);
                }
            }
        } else if key == "DELETE" {
            let len = current_input.get().len(); if len > 0 { set_last_typed_index.set(len as i32 - 1); set_destroy_trigger.set(js_sys::Date::now().to_string()); set_timeout(move || { set_current_input.update(|s| { s.pop(); }); set_last_typed_index.set(-1); set_destroy_trigger.set("".to_string()); }, std::time::Duration::from_millis(150)); }
        } else if current_input.get().len() < 5 { let next_idx = current_input.get().len() as i32; set_last_typed_index.set(next_idx); let k = key.to_uppercase(); set_keyboard_pulse.set((k.chars().next().unwrap(), js_sys::Date::now().to_string())); set_current_input.update(|s| s.push_str(&k)); }
    };

    let _ = window_event_listener(leptos::ev::keydown, move |ev| { 
        let key = ev.key(); 
        if show_stats.get() {
            if key == "Enter" && (game_won.get() || game_lost.get()) {
                share_results();
            }
            return; 
        }
        if show_help.get() { return; } 
        if key == "Enter" { on_key("ENTER".to_string()); } else if key == "Backspace" { on_key("DELETE".to_string()); } else if key.len() == 1 { let c = key.chars().next().unwrap(); if c.is_ascii_alphabetic() { on_key(c.to_uppercase().to_string()); } } 
    });

    view! {
        <div class="flex flex-col h-full bg-app-bg text-app-text overflow-hidden transition-all duration-500 relative">
            <div class="absolute top-2 right-2 text-[8px] font-mono opacity-30 pointer-events-none z-50">"v1.3.0"</div>
            <header class="w-full flex flex-col items-center pt-2 sm:pt-4 shrink-0 relative z-50">
                <div class="flex items-center gap-3">
                    <h1 class="text-3xl sm:text-5xl font-black tracking-tighter italic text-center title-text uppercase">"RUSTLE"</h1>
                    <button on:click=move |_| start_ng_plus() class=move || format!("w-8 h-8 sm:w-10 sm:h-10 flex items-center justify-center rounded-xl shadow-lg border-2 transition-all active:scale-90 {}", if is_ng_plus.get() || daily_game_done.get() { "ai-active-pad border-transparent shadow-[0_0_15px_rgba(255,0,255,0.8)]" } else { "cell-neutral border-current opacity-40 hover:opacity-100" })> <span class=move || format!("text-xl sm:text-2xl font-black {}", if is_ng_plus.get() || daily_game_done.get() { "text-white" } else { "text-current opacity-50" })>"+"</span> </button>
                </div>
            </header>
            
            <main class="flex-1 flex flex-col w-full max-w-2xl mx-auto px-2 sm:px-4 min-h-0 relative">
                <div class="flex-1 flex items-end justify-center pointer-events-none px-4 relative z-50 pb-2"> {move || { let snark = snarky_comment.get(); if !snark.is_empty() { let color = if game_won.get() { "text-green-400" } else if game_lost.get() { "text-red-400" } else { "text-theme-primary" }; view! { <div key=snark.clone() class=format!("text-[10px] sm:text-xs font-black uppercase tracking-widest {} snarky-toast text-center pointer-events-auto", color)> {snark} </div> }.into_view() } else { view! {}.into_view() } }} </div>
                
                <div class="w-full flex justify-center relative">
                    <aside class="flex flex-col gap-4 py-4 shrink-0 absolute left-0 sm:left-4 scale-75 sm:scale-100 origin-left z-40">
                    <button on:click=move |_| { global_stats_res.refetch(); set_show_stats.set(true); } title="Team Status" class="btn-large correct-pad shadow-lg border-2 border-transparent transition-all active:scale-90"> <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path></svg> </button>
                    <button on:click=move |_| set_show_help.set(true) title="How to Play" class="btn-large correct-pad shadow-lg border-2 border-transparent transition-all active:scale-90"> <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg> </button>
                    <button on:click=move |_| {
                        if is_ng_plus.get() {
                            let msgs = ["NO ESCAPE FROM THE SYSTEM.", "MANDATORY HARD MODE.", "SUCK IT UP, BUTTERCUP.", "YOU ASKED FOR THIS.", "AI DOES NOT ALLOW COWARDICE."];
                            set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                            set_timeout(move || set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                        } else if guesses.get().is_empty() {
                            set_hard_mode.update(|h| *h = !*h);
                            let msgs = if hard_mode.get() { ["A GLUTTON FOR PUNISHMENT.", "OH, YOU THINK YOU'RE SMART?", "BRING THE PAIN.", "HARD MODE ENGAGED.", "NO MERCY.", "PREPARE TO SUFFER.", "FINALLY, A CHALLENGE."] } else { ["COWARD.", "TOO HARD FOR YOU?", "BACK TO BABY MODE.", "COPPING OUT ALREADY?", "WEAK AURA.", "I EXPECTED BETTER.", "EASY MODE ENGAGED."] };
                            set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                            set_timeout(move || set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                        } else {
                            let msgs = ["TOO LATE TO CHANGE NOW.", "YOU MADE YOUR BED.", "NO BACKING OUT MID-GAME.", "COMMITTED TO THIS PATH.", "NICE TRY."];
                            set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                            set_timeout(move || set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                        }
                    } title="Hard Mode" class=move || format!("btn-large shadow-lg border-2 transition-all active:scale-90 {}", if hard_mode.get() || is_ng_plus.get() { "correct-pad border-transparent" } else { "cell-neutral border-current" })> <svg class=move || format!("w-6 h-6 sm:w-8 sm:h-8 transition-all {}", if hard_mode.get() || is_ng_plus.get() { "text-yellow-300 scale-110 drop-shadow-[0_0_12px_rgba(253,224,71,1)]" } else { "text-current opacity-40" }) fill="currentColor" viewBox="0 0 24 24"> <path d="M13 10V3L4 14h7v7l9-11h-7z"></path> </svg> </button>
                </aside>
                <div class="flex-1 flex flex-col items-center justify-center min-h-0 py-2 px-10 sm:px-8 relative z-30"> <div class="flex flex-col gap-1 sm:gap-2 h-full max-h-[480px] aspect-[5/6]"> {move || { let gs = guesses.get(); let ss = guess_statuses.get(); let is_rev = is_revealing_row.get(); let len = gs.len(); let hard = hard_mode.get() || is_ng_plus.get(); gs.into_iter().zip(ss.into_iter()).enumerate().map(move |(i, (g, s))| { view! { <Row guess=g.to_uppercase() statuses=s is_revealing=is_rev && i == len-1 is_completed=true is_jiggling=Signal::derive(|| false) destroy_trigger=destroy_trigger last_typed_index=-1 is_hard_mode=hard /> } }).collect_view() }} {move || if guesses.get().len() < 6 && !game_won.get() { let input = current_input.get().to_uppercase(); let last = last_typed_index.get(); let hard = hard_mode.get() || is_ng_plus.get(); view! { <Row guess=input statuses=vec![] is_revealing=false is_completed=false is_jiggling=Signal::derive(move || jiggle_row.get()) destroy_trigger=destroy_trigger last_typed_index=last is_hard_mode=hard /> }.into_view() } else { view! {}.into_view() }} {move || { let hard = hard_mode.get() || is_ng_plus.get(); (0..(6_usize.saturating_sub(guesses.get().len() + if guesses.get().len() < 6 && !game_won.get() { 1 } else { 0 }))).map(move |_| { view! { <Row guess="".to_string() statuses=vec![] is_revealing=false is_completed=false is_jiggling=Signal::derive(|| false) destroy_trigger=destroy_trigger last_typed_index=-1 is_hard_mode=hard /> } }).collect_view() }} </div> </div>
                <aside class="flex flex-col gap-3 py-4 shrink-0 absolute right-0 sm:right-4 scale-75 sm:scale-100 origin-right z-40"> {move || {
                    let themes = [("red", "bg-red-600 text-white", "Red"), ("orange", "bg-orange-500 text-white", "Orange"), ("yellow", "bg-yellow-400 text-black", "Yellow"), ("green", "bg-green-600 text-white", "Green"), ("blue", "bg-blue-600 text-white", "Blue")];
                    let cur = theme.get(); let pulse = win_pulse_trigger.get();
                    let s = global_stats_res.get().unwrap_or_default();
                    themes.into_iter().map(move |(t, bg, label)| {
                        let t_str = t.to_string(); let is_act = cur == t; let pulse_val = pulse.clone(); let pulse_val_2 = pulse.clone();
                        let winner_val = s.yesterday_winner.clone();
                        view! { <button on:click=move |_| {
                            if theme.get() != t_str {
                                set_theme.set(t_str.clone());
                                post_score(t_str.clone(), 0);
                                let l = label.to_uppercase();
                                let msgs = match t_str.as_str() {
                                    "red" => vec!["SKIBIDI.".to_string(), "NO CAP.".to_string(), "GOAT STATUS.".to_string(), "RIZZ.".to_string(), "SIGMA GRINDSET.".to_string()],
                                    "orange" => vec!["VIBE CHECK.".to_string(), "MAIN CHARACTER ENERGY.".to_string(), "TOUCH GRASS.".to_string(), "LITERAL CHILLS.".to_string(), "IT'S GIVING WINNER.".to_string()],
                                    "yellow" => vec!["ADULTING IS HARD.".to_string(), "PUPPERCINO VIBES.".to_string(), "DOING THE BARE MINIMUM.".to_string(), "SLAY YAAAAS.".to_string(), "AVOCADO TOAST SECURED.".to_string()],
                                    "green" => vec!["WHATEVER.".to_string(), "AS IF.".to_string(), "RADICAL DEPARTURE.".to_string(), "STAY GOLDEN.".to_string(), "NOT IMPRESSED.".to_string()],
                                    "blue" => vec!["BACK IN MY DAY...".to_string(), "RESPECT YOUR ELDERS.".to_string(), "KIDS THESE DAYS.".to_string(), "GET OFF MY LAWN.".to_string(), "PULL YOURSELF BY THE BOOTSTRAPS.".to_string()],
                                    _ => vec![format!("JOINING TEAM {}.", l)]
                                };
                                set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                                set_timeout(move || set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                            } else {
                                let msgs = match t_str.as_str() {
                                    "red" => vec!["BRO STOP CAPPING.", "SKIBIDI SPAM.", "LITERALLY RENT FREE.", "AI IS CRINGING.", "TOUCH GRASS ALREADY."],
                                    "orange" => vec!["WE GET IT, BESTIE.", "NOT VERY MINDFUL CLICKING.", "IT'S THE SPAM FOR ME.", "STOP, I'M LITERALLY SHAKING.", "BIG YIKES."],
                                    "yellow" => vec!["STOP POKING ME.", "I CAN'T EVEN WITH THIS.", "SIRI, HOW TO STOP SPAM?", "DOING THE MOST.", "NARCISSIST MUCH?"],
                                    "green" => vec!["TAKE A CHILL PILL.", "GAG ME WITH A SPOON.", "STOP HARSHING MY MELLOW.", "YOU'RE TRIPPING.", "BOGUS SPAM."],
                                    "blue" => vec!["STOP THAT RACKET.", "PATIENCE IS A VIRTUE.", "KIDS AND THEIR BUTTONS.", "IN MY DAY WE CLICKED ONCE.", "HOLD YOUR HORSES."],
                                    _ => vec!["WE GET IT.", "YOU ARE ALREADY HERE.", "STOP CLICKING THAT."]
                                };
                                set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                                set_timeout(move || set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                            }
                        } title=format!("{} Team", label) class=format!("theme-square {} active:scale-125 {}", bg, if is_act { "active ring-2 ring-white ring-offset-2" } else { "" })> <Show when=move || winner_val == t> <div class="crown-icon">"👑"</div> </Show> <Show when=move || is_act> <div class="absolute inset-0 flex items-center justify-center font-black drop-shadow-md z-10 text-[10px] sm:text-xs text-white">{session_points.get()}</div> </Show> <Show when=move || !pulse_val.is_empty() && is_act> <div key=pulse_val_2.clone() class="win-pulse">{pulse_val_2.clone()}</div> </Show> </button> }
                    }).collect_view()
                }} </aside>
                </div>
                <div class="flex-1"></div>
            </main>
            <footer class="w-full max-w-2xl mx-auto p-2 pb-6 shrink-0 relative z-50"> {move || { let rows: &[&[char]] = &[&['Q','W','E','R','T','Y','U','I','O','P'], &['A','S','D','F','G','H','J','K','L'], &['Z','X','C','V','B','N','M']]; rows.iter().enumerate().map(|(i, row)| { view! { <div class="flex justify-center mb-1.5 w-full gap-1 sm:gap-1.5 px-1"> {if i == 2 { let is_full = current_input.get().len() == 5; view! { <button class=format!("key-large px-3 rounded-xl font-black transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg flex-[1.5] {} flex items-center justify-center", if is_full { "bg-white text-black shadow-[0_0_15px_rgba(255,255,255,0.8)] scale-105" } else { "key-neutral" }) on:click=move |_| on_key("ENTER".to_string())> <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path d="M13 7l5 5m0 0l-5 5m5-5H6"></path></svg> </button> }.into_view() } else { view! {}.into_view() }} {row.iter().map(|c| { let status = move || char_statuses.get().get(c).cloned().unwrap_or_default(); let pulse = move || if keyboard_pulse.get().0 == *c { keyboard_pulse.get().1 } else { "".to_string() }; let hard = hard_mode.get() || is_ng_plus.get(); view! { <button class=move || format!("key-large relative rounded-xl font-black flex-1 min-w-[30px] transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg border-2 border-transparent {}", match status().as_str() { "correct" => "correct", "present" => "present", "absent" => "absent", _ => "key-neutral" }) on:click=move |_| on_key(c.to_string())> {move || { let id = pulse(); if !id.is_empty() { if hard { view! { <div key=id class="power-ring" /> }.into_view() } else { view! { <div key=id class="power-underline" /> }.into_view() } } else { view! {}.into_view() } }} {c.to_string()} </button> } }).collect_view()} {if i == 2 { view! { <button class="key-large px-3 rounded-xl font-black transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg flex-[1.5] key-neutral flex items-center justify-center" on:click=move |_| on_key("DELETE".to_string())> <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path d="M12 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2M3 12l6.414 6.414a2 2 0 001.414.586H19a2 2 0 002-2V7a2 2 0 00-2-2h-8.172a2 2 0 00-1.414.586L3 12z"></path></svg> </button> }.into_view() } else { view! {}.into_view() }} </div> } }).collect_view() }} </footer>
            
            <Modal title="Leaderboard".to_string() is_open=show_stats set_is_open=set_show_stats> <div class="flex flex-col items-center text-center text-white p-2 rounded-xl bg-black bg-opacity-40"> <div class="w-full mb-6 text-center"> <h3 class="text-xs font-bold uppercase mb-4 tracking-widest text-theme-primary">"TEAMS"</h3> <div class="grid grid-cols-5 w-full gap-2 px-2"> {move || { let gs = global_stats_res.get().unwrap_or_default(); let team_data = [("red", "bg-red-600 text-white", "R"), ("orange", "bg-orange-500 text-white", "O"), ("yellow", "bg-yellow-400 text-black", "Y"), ("green", "bg-green-600 text-white", "G"), ("blue", "bg-blue-600 text-white", "B")]; team_data.into_iter().map(move |(t, bg, label)| { let d = match t { "yellow" => &gs.yellow, "red" => &gs.red, "green" => &gs.green, "blue" => &gs.blue, _ => &gs.orange }; let win_name = gs.yesterday_winner.clone(); view! { <div class="flex flex-col items-center"> <div class=format!("theme-square {} active ring-2 ring-white ring-opacity-20 mb-1", bg)> <Show when=move || win_name == t> <div class="crown-icon">"👑"</div> </Show> <div class="text-[10px] font-black">{d.points}</div> </div> <div class="text-[6px] uppercase opacity-50 font-black">{label}</div> <div class="text-[5px] uppercase opacity-30 mb-1">{d.players} "PLYRS"</div> </div> } }).collect_view() }} </div> </div> <div class="w-full border-t border-white border-opacity-10 pt-6 mb-6"> <h3 class="text-xs font-bold uppercase mb-4 tracking-widest text-theme-primary text-center">"Individual Ranking"</h3> <div class="grid grid-cols-4 w-full gap-4 mb-8"> <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().total_games}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Played"</div></div> <div class="flex flex-col"><div class="text-3xl font-black">{move || if stats.get().total_games > 0 { stats.get().wins * 100 / stats.get().total_games } else { 0 }}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Win %"</div></div> <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().current_streak}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Streak"</div></div> <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().best_streak}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Best"</div></div> </div> <h3 class="text-xs font-bold uppercase mb-4 tracking-widest text-theme-primary text-center">"Guess Distribution"</h3> <div class="w-full space-y-1.5 text-left"> {move || stats.get().distribution.iter().enumerate().map(|(i, count)| { let wins = stats.get().wins; let width = if wins > 0 { (*count as f32 * 100.0 / wins as f32).max(10.0) } else { 10.0 }; view! { <div class="flex items-center gap-2 text-xs text-white"><div class="w-2">{i+1}</div><div class="bg-gray-600 text-white font-black p-0.5 rounded text-right pr-2 transition-all duration-1000" style=format!("width: {}%", width)>{*count}</div></div> } }).collect_view()} </div> </div> <Show when=move || game_won.get() || game_lost.get()> <button on:click=move |_| share_results() class="w-full mt-4 bg-green-500 hover:bg-green-400 text-white font-black py-3 rounded-xl shadow-[0_0_15px_rgba(34,197,94,0.8)] transition-all active:scale-95 uppercase tracking-widest animate-pulse"> "SHARE & CONTINUE" </button> </Show> </div> </Modal>
            
            <Modal title="How to Play".to_string() is_open=show_help set_is_open=set_show_help> <div class="flex flex-col gap-6 text-white text-center"> <div class="space-y-4"> <div class="flex flex-col items-center gap-1"> <div class="flex"> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent correct font-black">"R"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"U"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"T"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div> </div> <div class="text-[10px] opacity-70">"R is in the correct spot."</div> </div> <div class="flex flex-col items-center gap-1"> <div class="flex"> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral font-bold">"W"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent present mx-0.5 font-black text-black">"O"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"R"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"D"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div> </div> <div class="text-[10px] opacity-70">"O is in the word but wrong spot."</div> </div> <div class="flex flex-col items-center gap-1"> <div class="flex"> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral font-bold">"V"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"A"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"G"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent absent mx-0.5 font-black">"U"</div> <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"E"</div> </div> <div class="text-[10px] opacity-70">"U is not in the word."</div> </div> </div> <div class="w-full border-t border-white border-opacity-10 pt-4 flex flex-col gap-4"> <div> <h3 class="text-xs font-bold uppercase mb-2 tracking-widest text-theme-primary">"Hard Mode ⚡"</h3> <p class="text-[10px] opacity-80 leading-relaxed">"Any revealed hints must be used in subsequent guesses. You cannot guess a word that does not contain the green and yellow letters you've already found. Earns double points."</p> </div> <div> <div class="flex items-center justify-center gap-1.5 mb-2"> <h3 class="text-xs font-bold uppercase tracking-widest text-theme-primary m-0">"New Game"</h3> <div class="ai-active-pad w-4 h-4 flex items-center justify-center rounded shadow-lg border border-transparent shadow-[0_0_10px_rgba(255,0,255,0.8)]"><span class="text-[10px] font-black text-white">"+"</span></div> </div> <p class="text-[10px] opacity-80 leading-relaxed">"Beat the daily game to unlock New Game+. In this mode, the game plays actively against you. There is no single pre-picked word. Instead, the AI dynamically dodges your guesses by switching the answer to whatever remaining valid word forces you to take the longest possible path to win. Hard Mode is strictly enforced. Good luck."</p> </div> </div> </div> </Modal>
        </div>
    }
}

fn main() { console_error_panic_hook::set_once(); let root = document().get_element_by_id("root").expect("could not find #root element"); mount_to(root.unchecked_into(), || view! { <App/> }) }
