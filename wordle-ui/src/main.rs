use leptos::*;
use wordle_engine::{is_word_in_list, get_guess_statuses};

#[component]
fn App() -> impl IntoView {
    let (current_guess, set_current_guess) = create_signal(String::new());
    let (guesses, set_guesses) = create_signal(Vec::<String>::new());
    
    view! {
        <div class="flex h-screen flex-col items-center justify-center bg-gray-900 text-white">
            <h1 class="text-4xl font-bold mb-8">"Wordle Rust (Leptos Edition)"</h1>
            <p>"The UI is successfully rendering via Rust and WebAssembly!"</p>
            
            <div class="mt-8 flex gap-2">
                <input 
                    type="text" 
                    class="p-2 rounded text-black"
                    on:input=move |ev| set_current_guess.set(event_target_value(&ev))
                    prop:value=current_guess
                />
                <button 
                    class="bg-green-500 hover:bg-green-600 p-2 rounded font-bold"
                    on:click=move |_| {
                        let g = current_guess.get();
                        if g.len() == 5 && is_word_in_list(&g) {
                            set_guesses.update(|gs| gs.push(g.clone()));
                            set_current_guess.set(String::new());
                        }
                    }
                >
                    "Guess"
                </button>
            </div>
            
            <div class="mt-8 flex flex-col gap-2">
                {move || guesses.get().into_iter().map(|g| {
                    let statuses: Vec<String> = serde_wasm_bindgen::from_value(get_guess_statuses("APPLE", &g)).unwrap_or_default();
                    view! {
                        <div class="flex gap-2 text-2xl font-bold uppercase">
                            {g.chars().zip(statuses.into_iter()).map(|(c, status): (char, String)| {
                                let bg_color = match status.as_str() {
                                    "correct" => "bg-green-500",
                                    "present" => "bg-yellow-500",
                                    _ => "bg-gray-600",
                                };
                                view! {
                                    <div class=format!("w-14 h-14 flex items-center justify-center rounded {}", bg_color)>
                                        {c}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
