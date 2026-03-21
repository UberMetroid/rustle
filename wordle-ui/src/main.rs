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
fn Cell(value: char, status: String, position: usize, is_revealing: bool) -> impl IntoView {
    let delay = position * 350;
    let classes = move || {
        let mut base = "w-12 h-12 sm:w-14 sm:h-14 border-solid border-2 flex items-center justify-center mx-0.5 text-2xl sm:text-4xl font-bold rounded transition-all duration-300".to_string();
        if !status.is_empty() { base.push_str(&format!(" {}", status)); }
        if is_revealing { base.push_str(" cell-reveal"); }
        base
    };
    let style = move || format!("animation-delay: {}ms;", delay);
    view! { <div class=classes style=style><div>{value.to_uppercase().to_string()}</div></div> }
}

#[component]
fn Row(guess: String, solution: String, is_revealing: bool) -> impl IntoView {
    let statuses: Vec<String> = if !guess.is_empty() && !solution.is_empty() {
        serde_wasm_bindgen::from_value(get_guess_statuses(&solution, &guess)).unwrap_or_default()
    } else {
        vec!["".to_string(); 5]
    };
    view! {
        <div class="flex justify-center mb-1">
            {guess.chars().chain(std::iter::repeat(' ')).take(5).zip(statuses.into_iter().chain(std::iter::repeat("".to_string()))).enumerate().map(|(i, (c, s))| {
                view! { <Cell value=c status=s position=i is_revealing=is_revealing /> }
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
            <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50">
                <div class="glass w-full max-w-sm rounded-2xl p-6 shadow-2xl transition-all scale-up border border-gray-500 border-opacity-30">
                    <div class="flex justify-between items-center mb-4 text-white">
                        <h2 class="text-2xl font-black tracking-tighter"> {title_clone.clone()} </h2>
                        <button on:click=move |_| set_is_open.set(false) class="text-2xl font-bold hover:text-red-500"> "×" </button>
                    </div>
                    <div class="text-white">
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
    
    let (theme, set_theme) = create_signal(
        window().local_storage().unwrap().unwrap()
            .get_item("color-theme").unwrap_or_default()
            .unwrap_or_else(|| "default".to_string())
    );
    
    let now = js_sys::Date::now();
    let solution_data = create_memo(move |_| {
        let val = get_solution(now as u64);
        let s: SolutionData = serde_wasm_bindgen::from_value(val).unwrap();
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
        let sol = solution_data.get().solution;
        for g in guesses.get() {
            let statuses: Vec<String> = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &g)).unwrap_or_default();
            for (c, s) in g.chars().zip(statuses.into_iter()) {
                let current = map.entry(c).or_insert(s.clone());
                if s == "correct" { *current = s; }
                else if s == "present" && *current != "correct" { *current = s; }
            }
        }
        map
    });

    let on_key = Callback::new(move |key: String| {
        if game_won.get() || game_lost.get() { return; }
        let storage = window().local_storage().unwrap().unwrap();

        if key == "ENTER" {
            let input = current_input.get();
            let sol = solution_data.get().solution;
            if input.len() == 5 && is_word_in_list(&input) {
                let mut new_guesses = guesses.get();
                new_guesses.push(input.clone());
                set_guesses.set(new_guesses.clone());
                set_current_input.set(String::new());
                
                // Save state
                let state = StoredState { guesses: new_guesses.clone(), solution: sol.clone() };
                let _ = storage.set_item("game-state", &serde_json::to_string(&state).unwrap());

                if input == sol {
                    set_game_won.set(true);
                    confetti();
                    set_stats.update(|s| {
                        s.total_games += 1;
                        s.wins += 1;
                        s.current_streak += 1;
                        if s.current_streak > s.best_streak { s.best_streak = s.current_streak; }
                        s.distribution[new_guesses.len() - 1] += 1;
                    });
                    let _ = storage.set_item("game-stats", &serde_json::to_string(&stats.get()).unwrap());
                    set_show_stats.set(true);
                } else if new_guesses.len() >= 6 {
                    set_game_lost.set(true);
                    set_stats.update(|s| {
                        s.total_games += 1;
                        s.current_streak = 0;
                    });
                    let _ = storage.set_item("game-stats", &serde_json::to_string(&stats.get()).unwrap());
                    set_show_stats.set(true);
                }
            }
        } else if key == "DELETE" {
            set_current_input.update(|s| { s.pop(); });
        } else if current_input.get().len() < 5 {
            set_current_input.update(|s| s.push_str(&key));
        }
    });

    let on_share = move |_| {
        let sol = solution_data.get().solution;
        let mut text = format!("Wordle Rust {} {}/6\n\n", solution_data.get().solution_index, if game_won.get() { guesses.get().len().to_string() } else { "X".to_string() });
        for g in guesses.get() {
            let statuses: Vec<String> = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &g)).unwrap_or_default();
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
    };

    create_effect(move |_| {
        let t = theme.get();
        let el = document().document_element().unwrap();
        let _ = el.set_attribute("class", &format!("theme-{}", t));
        let _ = window().local_storage().unwrap().unwrap().set_item("color-theme", &t);
    });

    view! {
        <div class="flex h-screen flex-col items-center justify-between py-4 sm:py-8 overflow-hidden transition-all duration-500 text-black dark:text-white">
            <div class="w-full max-w-[500px] flex flex-col items-center">
                <nav class="w-full flex justify-between items-center px-4 mb-4 sm:mb-8 border-b border-gray-500 border-opacity-30 pb-2 glass rounded-b-xl shadow-lg">
                    <button on:click=move |_| set_show_stats.set(true)>
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path></svg>
                    </button>
                    <h1 class="text-2xl sm:text-3xl font-black tracking-tighter">"RUSTLE"</h1>
                    <div class="flex gap-1.5">
                        {vec!["retro", "cyberpunk", "nord", "default", "solarized"].into_iter().map(|t| {
                            let color = match t {
                                "retro" => "#00ff41", "cyberpunk" => "#ff007f", "nord" => "#88c0d0", "default" => "#10b981", _ => "#b58900",
                            };
                            view! {
                                <button class=move || format!("w-5 h-5 rounded-full border-2 transition-all hover:scale-125 {}", if theme.get() == t { "border-white scale-110 shadow-lg" } else { "border-gray-500" })
                                    style=format!("background-color: {}", color) on:click=move |_| set_theme.set(t.to_string()) />
                            }
                        }).collect_view()}
                    </div>
                </nav>

                <div class="flex flex-col gap-1 sm:gap-2 px-2">
                    {move || guesses.get().into_iter().map(|g| { view! { <Row guess=g solution=solution_data.get().solution is_revealing=true /> } }).collect_view()}
                    {move || if guesses.get().len() < 6 && !game_won.get() { view! { <Row guess=current_input.get() solution=solution_data.get().solution is_revealing=false /> }.into_view() } else { view! {}.into_view() }}
                    {move || (0..(6_usize.saturating_sub(guesses.get().len() + if guesses.get().len() < 6 && !game_won.get() { 1 } else { 0 }))).map(|_| { view! { <Row guess="".to_string() solution="".to_string() is_revealing=false /> } }).collect_view()}
                </div>
            </div>

            <div class="mt-4 w-full max-w-[500px] px-2 flex flex-col items-center">
                {move || {
                    let rows = vec![vec!['Q','W','E','R','T','Y','U','I','O','P'], vec!['A','S','D','F','G','H','J','K','L'], vec!['Z','X','C','V','B','N','M']];
                    rows.into_iter().enumerate().map(|(i, row)| {
                        view! {
                            <div class="flex justify-center mb-2 w-full">
                                {if i == 2 { view! { <button class="h-12 sm:h-14 px-2 mx-0.5 rounded font-bold bg-gray-400 text-white flex-[1.5] text-xs" on:click=move |_| on_key.call("ENTER".to_string())> "ENTER" </button> }.into_view() } else { view! {}.into_view() }}
                                {row.into_iter().map(|c| {
                                    let status = move || char_statuses.get().get(&c).cloned().unwrap_or_default();
                                    let bg = move || match status().as_str() { "correct" => "bg-green-500", "present" => "bg-yellow-500", "absent" => "bg-gray-700", _ => "bg-gray-400" };
                                    view! { <button class=move || format!("h-12 sm:h-14 mx-0.5 rounded font-bold text-white flex-1 min-w-[25px] transition-colors duration-500 {}", bg()) on:click=move |_| on_key.call(c.to_string())> {c.to_string()} </button> }
                                }).collect_view()}
                                {if i == 2 { view! { <button class="h-12 sm:h-14 px-2 mx-0.5 rounded font-bold bg-gray-400 text-white flex-[1.5] text-xs" on:click=move |_| on_key.call("DELETE".to_string())> "DEL" </button> }.into_view() } else { view! {}.into_view() }}
                            </div>
                        }
                    }).collect_view()
                }}
            </div>

            <Modal title="Statistics".to_string() is_open=show_stats set_is_open=set_show_stats>
                <div class="flex flex-col items-center text-center">
                    <div class="flex w-full justify-around mb-6">
                        <div><div class="text-3xl font-black">{move || stats.get().total_games}</div><div class="text-xs uppercase opacity-70">"Played"</div></div>
                        <div><div class="text-3xl font-black">{move || if stats.get().total_games > 0 { stats.get().wins * 100 / stats.get().total_games } else { 0 }}</div><div class="text-xs uppercase opacity-70">"Win %"</div></div>
                        <div><div class="text-3xl font-black">{move || stats.get().current_streak}</div><div class="text-xs uppercase opacity-70">"Streak"</div></div>
                        <div><div class="text-3xl font-black">{move || stats.get().best_streak}</div><div class="text-xs uppercase opacity-70">"Best"</div></div>
                    </div>
                    <h3 class="text-sm font-bold uppercase mb-2">"Guess Distribution"</h3>
                    <div class="w-full space-y-1 mb-6 text-left">
                        {move || stats.get().distribution.iter().enumerate().map(|(i, count)| {
                            let wins = stats.get().wins;
                            let width = if wins > 0 { (*count as f32 * 100.0 / wins as f32).max(10.0) } else { 10.0 };
                            view! { <div class="flex items-center gap-2 text-xs"><div class="w-2">{i+1}</div><div class="bg-gray-500 text-white font-bold p-0.5 rounded text-right pr-2 transition-all duration-1000" style=format!("width: {}%", width)>{*count}</div></div> }
                        }).collect_view()}
                    </div>
                    <Show when=move || game_won.get() || game_lost.get()>
                        <button on:click=on_share class="w-full bg-green-500 hover:bg-green-600 text-white font-black py-3 rounded-xl shadow-lg flex items-center justify-center gap-2 transition-all active:scale-95">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"></path></svg>
                            "SHARE RESULT"
                        </button>
                    </Show>
                </div>
            </Modal>

            {move || if game_won.get() { view! { <div class="fixed top-24 px-6 py-3 rounded-full bg-green-500 text-white font-black text-xl shadow-2xl animate-bounce glass"> "AMAZING! YOU WON!" </div> }.into_view() } 
              else if game_lost.get() { view! { <div class="fixed top-24 px-6 py-3 rounded-full bg-red-500 text-white font-black text-xl shadow-2xl glass uppercase font-black"> {format!("The word was: {}", solution_data.get().solution)} </div> }.into_view() } 
              else { view! {}.into_view() }}
        </div>
    }
}

fn main() { console_error_panic_hook::set_once(); mount_to_body(|| view! { <App/> }) }
fn document() -> web_sys::Document { leptos::document() }
