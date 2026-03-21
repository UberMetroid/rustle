use leptos::*;
use word_engine::{get_solution, is_word_in_list, get_guess_statuses};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

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

#[component]
fn Cell(value: char, status: String, position: usize, is_revealing: bool) -> impl IntoView {
    let delay = position * 350;
    let classes = move || {
        let mut base = "w-14 h-14 border-solid border-2 flex items-center justify-center mx-0.5 text-4xl font-bold rounded transition-all duration-300".to_string();
        if !status.is_empty() {
            base.push_str(&format!(" {}", status));
        }
        if is_revealing {
            base.push_str(" cell-reveal");
        }
        base
    };

    let style = move || {
        format!("animation-delay: {}ms;", delay)
    };

    view! {
        <div class=classes style=style>
            <div>{value.to_uppercase().to_string()}</div>
        </div>
    }
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
fn Keyboard(on_key: Callback<String>, char_statuses: Memo<std::collections::HashMap<char, String>>) -> impl IntoView {
    let rows = vec![
        vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
        vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
        vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
    ];

    view! {
        <div class="mt-4 w-full max-w-[500px] px-2">
            {rows.into_iter().enumerate().map(|(i, row)| {
                view! {
                    <div class="flex justify-center mb-2 w-full">
                        {if i == 2 {
                            view! {
                                <button 
                                    class="h-14 px-2 mx-0.5 rounded font-bold bg-gray-400 text-white flex-[1.5] text-xs sm:text-sm"
                                    on:click=move |_| on_key.call("ENTER".to_string())
                                >
                                    "ENTER"
                                </button>
                            }.into_view()
                        } else { view! {}.into_view() }}
                        
                        {row.into_iter().map(|c| {
                            let status = move || char_statuses.get().get(&c).cloned().unwrap_or_default();
                            let bg_class = move || match status().as_str() {
                                "correct" => "bg-green-500",
                                "present" => "bg-yellow-500",
                                "absent" => "bg-gray-700",
                                _ => "bg-gray-400",
                            };
                            view! {
                                <button 
                                    class=move || format!("h-14 mx-0.5 rounded font-bold text-white flex-1 min-w-[20px] sm:min-w-[30px] transition-colors duration-500 {}", bg_class())
                                    on:click=move |_| on_key.call(c.to_string())
                                >
                                    {c.to_string()}
                                </button>
                            }
                        }).collect_view()}

                        {if i == 2 {
                            view! {
                                <button 
                                    class="h-14 px-2 mx-0.5 rounded font-bold bg-gray-400 text-white flex-[1.5] text-xs sm:text-sm"
                                    on:click=move |_| on_key.call("DELETE".to_string())
                                >
                                    "DEL"
                                </button>
                            }.into_view()
                        } else { view! {}.into_view() }}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (guesses, set_guesses) = create_signal(Vec::<String>::new());
    let (current_input, set_current_input) = create_signal(String::new());
    let (game_won, set_game_won) = create_signal(false);
    let (game_lost, set_game_lost) = create_signal(false);
    
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

    let char_statuses = create_memo(move |_| {
        let mut map = std::collections::HashMap::new();
        let sol = solution_data.get().solution;
        for g in guesses.get() {
            let statuses: Vec<String> = serde_wasm_bindgen::from_value(get_guess_statuses(&sol, &g)).unwrap_or_default();
            for (c, s) in g.chars().zip(statuses.into_iter()) {
                let current = map.entry(c).or_insert(s.clone());
                if s == "correct" {
                    *current = s;
                } else if s == "present" && *current != "correct" {
                    *current = s;
                }
            }
        }
        map
    });

    let on_key = Callback::new(move |key: String| {
        if game_won.get() || game_lost.get() { return; }

        if key == "ENTER" {
            let input = current_input.get();
            let sol = solution_data.get().solution;
            if input.len() == 5 && is_word_in_list(&input) {
                set_guesses.update(|gs| gs.push(input.clone()));
                set_current_input.set(String::new());
                
                if input == sol {
                    set_game_won.set(true);
                    // Fancy Win state!
                    confetti();
                } else if guesses.get().len() >= 6 {
                    set_game_lost.set(true);
                }
            }
        } else if key == "DELETE" {
            set_current_input.update(|s| { s.pop(); });
        } else if current_input.get().len() < 5 {
            set_current_input.update(|s| s.push_str(&key));
        }
    });

    create_effect(move |_| {
        let t = theme.get();
        let el = document().document_element().unwrap();
        let _ = el.set_attribute("class", &format!("theme-{}", t));
        let _ = window().local_storage().unwrap().unwrap().set_item("color-theme", &t);
    });

    view! {
        <div class="flex h-screen flex-col items-center justify-between py-4 sm:py-8 overflow-hidden transition-all duration-500">
            <div class="w-full max-w-[500px] flex flex-col items-center">
                <nav class="w-full flex justify-between items-center px-4 mb-4 sm:mb-8 border-b border-gray-500 border-opacity-30 pb-2 glass rounded-b-xl shadow-lg">
                    <h1 class="text-2xl sm:text-3xl font-black tracking-tighter">"WORDLE RUST"</h1>
                    <div class="flex gap-1.5 sm:gap-2">
                        {vec!["retro", "cyberpunk", "nord", "default", "solarized"].into_iter().map(|t| {
                            let color = match t {
                                "retro" => "#00ff41",
                                "cyberpunk" => "#ff007f",
                                "nord" => "#88c0d0",
                                "default" => "#10b981",
                                _ => "#b58900",
                            };
                            view! {
                                <button 
                                    class=move || format!("w-5 h-5 sm:w-6 sm:h-6 rounded-full border-2 transition-all hover:scale-125 {}", 
                                        if theme.get() == t { "border-white scale-110 shadow-lg" } else { "border-gray-500" })
                                    style=format!("background-color: {}", color)
                                    on:click=move |_| set_theme.set(t.to_string())
                                    title=t
                                />
                            }
                        }).collect_view()}
                    </div>
                </nav>

                <div class="flex flex-col gap-1 sm:gap-2 px-2">
                    {move || guesses.get().into_iter().map(|g| {
                        view! { <Row guess=g solution=solution_data.get().solution is_revealing=true /> }
                    }).collect_view()}
                    
                    {move || if guesses.get().len() < 6 && !game_won.get() {
                        view! { <Row guess=current_input.get() solution=solution_data.get().solution is_revealing=false /> }.into_view()
                    } else { view! {}.into_view() }}

                    {move || (0..(6_usize.saturating_sub(guesses.get().len() + if guesses.get().len() < 6 && !game_won.get() { 1 } else { 0 }))).map(|_| {
                        view! { <Row guess="".to_string() solution="".to_string() is_revealing=false /> }
                    }).collect_view()}
                </div>
            </div>

            <Keyboard on_key=on_key char_statuses=char_statuses />
            
            // Floating Win/Loss Message
            {move || if game_won.get() {
                view! { 
                    <div class="fixed top-24 px-6 py-3 rounded-full bg-green-500 text-white font-black text-xl shadow-2xl animate-bounce glass">
                        "AMAZING! YOU WON!"
                    </div> 
                }.into_view()
            } else if game_lost.get() {
                view! { 
                    <div class="fixed top-24 px-6 py-3 rounded-full bg-red-500 text-white font-black text-xl shadow-2xl glass">
                        {format!("THE WORD WAS: {}", solution_data.get().solution)}
                    </div> 
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
