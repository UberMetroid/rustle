use leptos::*;
use word_engine::{get_solution, is_word_in_list, get_guess_statuses};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

// Re-exporting for convenience
mod word_engine {
    pub use wordle_engine::*;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = confetti)]
    fn confetti();
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct SolutionData {
    pub solution: String,
    #[serde(rename = "solutionGameDate")]
    pub solution_game_date: u64,
    #[serde(rename = "solutionIndex")]
    pub solution_index: i64,
    pub tomorrow: u64,
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
    pub solution: String,
}

#[component]
fn Cell(value: char, status: String, position: usize, is_revealing: bool, is_completed: bool) -> impl IntoView {
    let delay = position * 350;
    let classes = move || {
        let mut base = "w-12 h-12 sm:w-14 sm:h-14 border-solid border-2 flex items-center justify-center mx-0.5 text-2xl sm:text-4xl font-bold rounded-xl transition-all duration-300".to_string();
        
        if is_completed || is_revealing {
            if !status.is_empty() {
                base.push_str(&format!(" {}", status));
            } else {
                base.push_str(" cell-neutral");
            }
        } else {
            base.push_str(" cell-neutral");
        }
        
        if is_revealing {
            base.push_str(" cell-reveal");
        }
        base
    };
    let style = move || format!("animation-delay: {}ms;", delay);
    view! { <div class=classes style=style><div>{value.to_uppercase().to_string()}</div></div> }
}

#[component]
fn Row(guess: String, solution: String, is_revealing: bool, is_completed: bool, is_jiggling: Signal<bool>) -> impl IntoView {
    let statuses: Vec<String> = if is_completed || is_revealing {
        serde_wasm_bindgen::from_value(get_guess_statuses(&solution, &guess)).unwrap_or_default()
    } else {
        vec!["".to_string(); 5]
    };
    view! {
        <div class=move || format!("flex justify-center mb-1 {}", if is_jiggling.get() { "jiggle" } else { "" })>
            {guess.chars().chain(std::iter::repeat(' ')).take(5).zip(statuses.into_iter().chain(std::iter::repeat("".to_string()))).enumerate().map(|(i, (c, s))| {
                view! { <Cell value=c status=s position=i is_revealing=is_revealing is_completed=is_completed /> }
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
            <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50" on:click=move |_| set_is_open.set(false)>
                <div class="glass-pad w-full max-w-sm p-6 shadow-2xl transition-all scale-up overflow-y-auto max-h-[90vh]" on:click=move |ev| ev.stop_propagation()>
                    <div class="flex justify-between items-center mb-4 text-white uppercase">
                        <h2 class="text-2xl font-black tracking-tighter"> {title_clone.clone()} </h2>
                        <button on:click=move |_| set_is_open.set(false) class="text-2xl font-bold hover:text-red-500 transition-colors"> "×" </button>
                    </div>
                    <div>
                        {children.with_value(|children| children())}
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn App() -> impl IntoView {
    let (guesses, set_guesses) = create_signal(Vec::<String>::new());
    let (current_input, set_current_input) = create_signal(String::new());
    let (game_won, set_game_won) = create_signal(false);
    let (game_lost, set_game_lost) = create_signal(false);
    let (show_stats, set_show_stats) = create_signal(false);
    let (show_settings, set_show_settings) = create_signal(false);
    let (jiggle_row, set_jiggle_row) = create_signal(false);
    let (alert_message, set_alert_message) = create_signal(String::new());
    let (is_revealing_row, set_is_revealing_row) = create_signal(false);
    
    let (theme, set_theme) = create_signal(
        window().local_storage().unwrap().unwrap()
            .get_item("color-theme").unwrap_or_default()
            .unwrap_or_else(|| "dark".to_string())
    );

    let (hard_mode, set_hard_mode) = create_signal(
        window().local_storage().unwrap().unwrap()
            .get_item("hard-mode").unwrap_or_default()
            .map(|v| v == "true").unwrap_or(false)
    );
    
    let now = js_sys::Date::now();
    let solution_data = create_memo(move |_| {
        let val = get_solution(now as u64);
        let s: SolutionData = serde_wasm_bindgen::from_value(val).unwrap_or_else(|_| {
            SolutionData { solution: "APPLE".to_string(), solution_game_date: 0, solution_index: 0, tomorrow: 0 }
        });
        s
    });

    let storage = window().local_storage().unwrap().unwrap();
    let stats_stored = storage.get_item("game-stats").unwrap_or_default()
        .and_then(|s| serde_json::from_str::<GameStats>(&s).ok())
        .unwrap_or_default();
    let (stats, set_stats) = create_signal(stats_stored);

    // Initial Load
    create_effect(move |_| {
        let storage = window().local_storage().unwrap().unwrap();
        let sol = solution_data.get().solution;
        if let Ok(Some(saved)) = storage.get_item("game-state") {
            if let Ok(state) = serde_json::from_str::<StoredState>(&saved) {
                if state.solution == sol {
                    set_guesses.set(state.guesses.clone());
                    if state.guesses.contains(&sol) {
                        set_game_won.set(true);
                    } else if state.guesses.len() >= 6 {
                        set_game_lost.set(true);
                    }
                }
            }
        }
    });

    let char_statuses = create_memo(move |_| {
        let mut map = HashMap::new();
        let sol = solution_data.get().solution.to_uppercase();
        for g in guesses.get() {
            let guess_upper = g.to_uppercase();
            let statuses: Vec<String> = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &guess_upper)).unwrap_or_default();
            for (c, s) in guess_upper.chars().zip(statuses.into_iter()) {
                let current = map.entry(c).or_insert(s.clone());
                if s == "correct" { *current = s; }
                else if s == "present" && *current != "correct" { *current = s; }
                else if s == "absent" && *current != "correct" && *current != "present" { *current = s; }
            }
        }
        map
    });

    let show_alert = move |msg: String| {
        set_alert_message.set(msg);
        set_jiggle_row.set(true);
        set_timeout(move || {
            set_alert_message.set(String::new());
            set_jiggle_row.set(false);
        }, std::time::Duration::from_millis(2000));
    };

    let on_key = Callback::new(move |key: String| {
        if game_won.get() || game_lost.get() { return; }
        let storage = window().local_storage().unwrap().unwrap();

        if key == "ENTER" {
            let input = current_input.get().to_uppercase();
            let sol = solution_data.get().solution.to_uppercase();
            
            if input.len() < 5 {
                show_alert("NOT ENOUGH LETTERS".to_string());
                return;
            }

            if !is_word_in_list(&input) {
                show_alert("NOT IN WORD LIST".to_string());
                return;
            }

            // Hard Mode Logic
            if hard_mode.get() && !guesses.get().is_empty() {
                let prev_guess = guesses.get().last().cloned().unwrap().to_uppercase();
                let prev_statuses: Vec<String> = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &prev_guess)).unwrap_or_default();
                
                for (i, (c, s)) in prev_guess.chars().zip(prev_statuses.iter()).enumerate() {
                    if s == "correct" && input.chars().nth(i).unwrap() != c {
                        show_alert(format!("MUST USE {} IN SPOT {}", c, i + 1));
                        return;
                    }
                }

                for (c, s) in prev_guess.chars().zip(prev_statuses.iter()) {
                    if s == "present" && !input.contains(c) {
                        show_alert(format!("MUST CONTAIN {}", c));
                        return;
                    }
                }
            }

            let mut new_guesses = guesses.get();
            new_guesses.push(input.clone());
            set_guesses.set(new_guesses.clone());
            set_current_input.set(String::new());
            
            let state = StoredState { guesses: new_guesses.clone(), solution: sol.clone() };
            let _ = storage.set_item("game-state", &serde_json::to_string(&state).unwrap());

            set_is_revealing_row.set(true);
            set_timeout(move || set_is_revealing_row.set(false), std::time::Duration::from_millis(2000));

            if input == sol {
                set_game_won.set(true);
                set_timeout(move || confetti(), std::time::Duration::from_millis(1800));
                set_stats.update(|s| {
                    s.total_games += 1;
                    s.wins += 1;
                    s.current_streak += 1;
                    if s.current_streak > s.best_streak { s.best_streak = s.current_streak; }
                    s.distribution[new_guesses.len() - 1] += 1;
                });
                let _ = storage.set_item("game-stats", &serde_json::to_string(&stats.get()).unwrap());
                set_timeout(move || set_show_stats.set(true), std::time::Duration::from_millis(3000));
            } else if new_guesses.len() >= 6 {
                set_game_lost.set(true);
                set_stats.update(|s| {
                    s.total_games += 1;
                    s.current_streak = 0;
                });
                let _ = storage.set_item("game-stats", &serde_json::to_string(&stats.get()).unwrap());
                set_timeout(move || set_show_stats.set(true), std::time::Duration::from_millis(3000));
            }
        } else if key == "DELETE" {
            set_current_input.update(|s| { s.pop(); });
        } else if current_input.get().len() < 5 {
            set_current_input.update(|s| s.push_str(&key.to_uppercase()));
        }
    });

    let on_share = move |_| {
        let sol = solution_data.get().solution.to_uppercase();
        let mut text = format!("Rustle {} {}/6\n\n", solution_data.get().solution_index, if game_won.get() { guesses.get().len().to_string() } else { "X".to_string() });
        for g in guesses.get() {
            let guess_upper = g.to_uppercase();
            let statuses: Vec<String> = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &guess_upper)).unwrap_or_default();
            for s in statuses {
                text.push_str(match s.as_str() {
                    "correct" => "🟩",
                    "present" => "🟨",
                    _ => "⬛",
                });
            }
            text.push('\n');
        }
        let _ = window().navigator().clipboard().write_text(&text);
        show_alert("COPIED TO CLIPBOARD".to_string());
    };

    create_effect(move |_| {
        let t = theme.get();
        let el = document().document_element().unwrap();
        let _ = el.set_attribute("class", &format!("theme-{}", t));
        let _ = window().local_storage().unwrap().unwrap().set_item("color-theme", &t);
    });

    create_effect(move |_| {
        let h = hard_mode.get();
        let _ = window().local_storage().unwrap().unwrap().set_item("hard-mode", if h { "true" } else { "false" });
    });

    view! {
        <div class="flex h-screen flex-col items-center justify-between py-4 sm:py-8 overflow-hidden transition-all duration-500 text-black dark:text-white px-2">
            <div class="w-full max-w-[600px] flex flex-col items-center">
                <nav class="w-full grid grid-cols-3 items-center px-4 mb-4 sm:mb-8">
                    <div class="flex gap-2 justify-start text-white">
                        <button on:click=move |_| set_show_stats.set(true) class="correct-pad w-12 h-12 sm:w-14 sm:h-14 flex items-center justify-center rounded-xl shadow-lg border-2 border-transparent">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path></svg>
                        </button>
                        <button on:click=move |_| set_show_settings.set(true) class="correct-pad w-12 h-12 sm:w-14 sm:h-14 flex items-center justify-center rounded-xl shadow-lg border-2 border-transparent">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.756 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.756 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
                        </button>
                    </div>
                    
                    <h1 class="text-2xl sm:text-4xl font-black tracking-tighter italic text-center title-text">"RUSTLE"</h1>
                    
                    <div class="flex justify-end">
                        <div class="glass-pad p-2 rounded-2xl flex items-center shadow-lg">
                            {move || {
                                let themes = vec!["dark", "red", "orange", "yellow", "green", "blue", "purple", "light"];
                                let current = theme.get();
                                let index = themes.iter().position(|&t| t == current).unwrap_or(0);
                                view! {
                                    <input 
                                        type="range" 
                                        min="0" 
                                        max="7" 
                                        step="1" 
                                        value=index
                                        class="theme-slider"
                                        on:input=move |ev| {
                                            let val = event_target_value(&ev).parse::<usize>().unwrap_or(0);
                                            set_theme.set(themes[val].to_string());
                                        }
                                    />
                                }
                            }}
                        </div>
                    </div>
                </nav>

                <div class="glass-pad p-4 sm:p-8">
                    <div class="flex flex-col gap-1 sm:gap-2">
                        {move || {
                            let gs = guesses.get();
                            let sol = solution_data.get().solution.to_uppercase();
                            let is_rev = is_revealing_row.get();
                            let len = gs.len();
                            gs.into_iter().enumerate().map(move |(i, g)| { 
                                view! { <Row guess=g.to_uppercase() solution=sol.clone() is_revealing=is_rev && i == len-1 is_completed=true is_jiggling=Signal::derive(|| false) /> } 
                            }).collect_view()
                        }}
                        
                        {move || if guesses.get().len() < 6 && !game_won.get() { 
                            let current_input = current_input.get().to_uppercase();
                            let solution = solution_data.get().solution.to_uppercase();
                            view! { <Row guess=current_input solution=solution is_revealing=false is_completed=false is_jiggling=Signal::derive(move || jiggle_row.get()) /> }.into_view() 
                        } else { view! {}.into_view() }}
                        
                        {move || (0..(6_usize.saturating_sub(guesses.get().len() + if guesses.get().len() < 6 && !game_won.get() { 1 } else { 0 }))).map(|_| { view! { <Row guess="".to_string() solution="".to_string() is_revealing=false is_completed=false is_jiggling=Signal::derive(|| false) /> } }).collect_view()}
                    </div>
                </div>
            </div>

            <div class="mt-4 w-full max-w-[550px] px-4 py-4 glass-pad flex flex-col items-center text-white shadow-2xl">
                {move || {
                    let rows = vec![vec!['Q','W','E','R','T','Y','U','I','O','P'], vec!['A','S','D','F','G','H','J','K','L'], vec!['Z','X','C','V','B','N','M']];
                    rows.into_iter().enumerate().map(|(i, row)| {
                        view! {
                            <div class="flex justify-center mb-2 w-full">
                                {if i == 2 { view! { 
                                    <button class="h-12 sm:h-14 px-2 mx-0.5 rounded-xl font-bold transition-all duration-500 hover:brightness-110 active:scale-95 shadow-lg flex-[1.5] flex items-center justify-center key-neutral" on:click=move |_| on_key.call("ENTER".to_string())> 
                                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path></svg>
                                    </button> 
                                }.into_view() } else { view! {}.into_view() }}
                                
                                {row.into_iter().map(|c| {
                                    let status = move || char_statuses.get().get(&c).cloned().unwrap_or_default();
                                    let status_class = move || match status().as_str() { 
                                        "correct" => "correct", 
                                        "present" => "present", 
                                        "absent" => "absent", 
                                        _ => "key-neutral" 
                                    };
                                    view! { <button class=move || format!("h-12 sm:h-14 mx-0.5 rounded-xl font-bold flex-1 min-w-[25px] transition-all duration-500 hover:brightness-110 active:scale-95 shadow-lg border-2 border-transparent {}", status_class()) on:click=move |_| on_key.call(c.to_string())> {c.to_string()} </button> }
                                }).collect_view()}
                                
                                {if i == 2 { view! { 
                                    <button class="h-12 sm:h-14 px-2 mx-0.5 rounded-xl font-bold transition-all duration-500 hover:brightness-110 active:scale-95 shadow-lg flex-[1.5] flex items-center justify-center key-neutral" on:click=move |_| on_key.call("DELETE".to_string())> 
                                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2M3 12l6.414 6.414a2 2 0 001.414.586H19a2 2 0 002-2V7a2 2 0 00-2-2h-8.172a2 2 0 00-1.414.586L3 12z"></path></svg>
                                    </button> 
                                }.into_view() } else { view! {}.into_view() }}
                            </div>
                        }
                    }).collect_view()
                }}
            </div>

            <Modal title="Statistics".to_string() is_open=show_stats set_is_open=set_show_stats>
                <div class="flex flex-col items-center text-center">
                    <div class="flex w-full justify-around mb-6 text-white text-white">
                        <div><div class="text-3xl font-black">{move || stats.get().total_games}</div><div class="text-xs uppercase opacity-70">"Played"</div></div>
                        <div><div class="text-3xl font-black">{move || if stats.get().total_games > 0 { stats.get().wins * 100 / stats.get().total_games } else { 0 }}</div><div class="text-xs uppercase opacity-70">"Win %"</div></div>
                        <div><div class="text-3xl font-black">{move || stats.get().current_streak}</div><div class="text-xs uppercase opacity-70">"Streak"</div></div>
                        <div><div class="text-3xl font-black">{move || stats.get().best_streak}</div><div class="text-xs uppercase opacity-70">"Best"</div></div>
                    </div>
                    <h3 class="text-sm font-bold uppercase mb-2 text-white">"Guess Distribution"</h3>
                    <div class="w-full space-y-1 mb-6 text-left">
                        {move || stats.get().distribution.iter().enumerate().map(|(i, count)| {
                            let wins = stats.get().wins;
                            let width = if wins > 0 { (*count as f32 * 100.0 / wins as f32).max(10.0) } else { 10.0 };
                            view! { <div class="flex items-center gap-2 text-xs text-white"><div class="w-2">{i+1}</div><div class="bg-gray-500 text-white font-bold p-0.5 rounded text-right pr-2 transition-all duration-1000" style=format!("width: {}%", width)>{*count}</div></div> }
                        }).collect_view()}
                    </div>
                    <Show when=move || game_won.get() || game_lost.get()>
                        <button on:click=on_share class="w-full bg-green-500 hover:bg-green-600 text-white font-black py-3 rounded-xl shadow-lg flex items-center justify-center gap-2 transition-all active:scale-95 uppercase">
                            "SHARE RESULT"
                        </button>
                    </Show>
                </div>
            </Modal>

            <Modal title="Settings".to_string() is_open=show_settings set_is_open=set_show_settings>
                <div class="flex flex-col gap-6 text-white">
                    <div class="flex justify-between items-center py-2 border-b border-gray-500 border-opacity-30">
                        <div>
                            <div class="font-bold">"Hard Mode"</div>
                            <div class="text-xs opacity-70">"Strict validation of clues"</div>
                            {move || if !guesses.get().is_empty() {
                                view! { <div class="text-[10px] text-red-400 mt-1 font-bold uppercase tracking-tighter italic">"Game in progress"</div> }.into_view()
                            } else {
                                view! {}.into_view()
                            }}
                        </div>
                        <button 
                            on:click=move |_| if guesses.get().is_empty() { set_hard_mode.update(|h| *h = !*h) }
                            class=move || format!("w-12 h-6 rounded-full transition-all duration-300 relative {} {}", 
                                if hard_mode.get() { "bg-green-500" } else { "bg-gray-600" },
                                if !guesses.get().is_empty() { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" }
                            )
                        >
                            <div class=move || format!("absolute top-1 w-4 h-4 bg-white rounded-full transition-all duration-300 {}", if hard_mode.get() { "left-7" } else { "left-1" }) />
                        </button>
                    </div>

                    // How to Play Examples
                    <div class="space-y-4">
                        <h3 class="text-sm font-black uppercase tracking-widest text-center opacity-80">"How to Play"</h3>
                        
                        <div class="space-y-3">
                            <div class="flex flex-col items-center gap-1">
                                <div class="flex">
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent correct font-black">"R"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"U"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"T"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                                </div>
                                <div class="text-[10px] opacity-70">"R is in the word and in the correct spot."</div>
                            </div>

                            <div class="flex flex-col items-center gap-1">
                                <div class="flex">
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral font-bold">"W"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent present mx-0.5 font-black">"O"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"R"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"D"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                                </div>
                                <div class="text-[10px] opacity-70">"O is in the word but in the wrong spot."</div>
                            </div>

                            <div class="flex flex-col items-center gap-1">
                                <div class="flex">
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral font-bold">"V"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"A"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"G"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent absent mx-0.5 font-black">"U"</div>
                                    <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"E"</div>
                                </div>
                                <div class="text-[10px] opacity-70">"U is not in the word in any spot."</div>
                            </div>
                        </div>
                    </div>

                    <div class="text-[10px] opacity-40 italic text-center mt-2">"Rustle Version 1.0.0 (Pure Rust)"</div>
                </div>
            </Modal>

            {move || if !alert_message.get().is_empty() {
                view! { <div class="fixed top-24 px-4 py-2 rounded-xl bg-black text-white font-bold shadow-xl glass z-[100] animate-bounce toast-slide text-center"> {alert_message.get()} </div> }.into_view()
            } else if game_won.get() {
                view! { <div class="fixed top-24 px-6 py-3 rounded-full bg-green-500 text-white font-black text-xl shadow-2xl animate-bounce glass text-center"> "AMAZING! YOU WON!" </div> }.into_view() 
            } else if game_lost.get() {
                view! { <div class="fixed top-24 px-6 py-3 rounded-full bg-red-500 text-white font-black text-xl shadow-2xl glass uppercase font-black text-center"> {format!("The word was: {}", solution_data.get().solution.to_uppercase())} </div> }.into_view() 
            } else { view! {}.into_view() }}
        </div>
    }
}

fn main() { console_error_panic_hook::set_once(); mount_to_body(|| view! { <App/> }) }
fn document() -> web_sys::Document { leptos::document() }
