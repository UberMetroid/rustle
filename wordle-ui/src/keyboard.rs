use leptos::*;
use crate::state::AppStateContext;

/// The on-screen keyboard component.
#[component]
pub fn Keyboard(
    /// Callback when a key is pressed.
    on_key: Callback<String>,
) -> impl IntoView {
    let state = use_context::<AppStateContext>().expect("AppStateContext not found");

    let rows: &[&[char]] = &[
        &['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
        &['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
        &['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
    ];

    view! {
        <footer class="w-full max-w-2xl mx-auto p-2 pb-safe shrink-0 relative z-50">
            {rows.iter().enumerate().map(|(i, row)| {
                view! {
                    <div class="flex justify-center mb-1.5 w-full gap-1 sm:gap-1.5 px-1">
                        {if i == 2 {
                            let is_full = move || state.current_input.get().len() == 5;
                            view! {
                                <button
                                    aria-label="Enter"
                                    class=move || format!(
                                        "key-large px-3 rounded-xl font-black transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg flex-[1.5] {} flex items-center justify-center",
                                        if is_full() { "bg-white text-black shadow-[0_0_15px_rgba(255,255,255,0.8)] scale-105" } else { "key-neutral" }
                                    )
                                    on:click=move |_| on_key.call("ENTER".to_string())
                                >
                                    <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
                                    </svg>
                                </button>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}

                        {row.iter().map(|c| {
                            let status = move || state.char_statuses.get().get(c).cloned().unwrap_or_default();
                            let pulse = move || if state.keyboard_pulse.get().0 == *c { state.keyboard_pulse.get().1 } else { "".to_string() };
                            let hard = move || state.hard_mode.get() || state.is_ng_plus.get();
                            
                            view! {
                                <button
                                    aria-label=format!("Letter {}", c)
                                    class=move || format!(
                                        "key-large relative rounded-xl font-black flex-1 min-w-[30px] transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg border-2 border-transparent {}",
                                        match status().as_str() {
                                            "correct" => "correct",
                                            "present" => "present",
                                            "absent" => "absent",
                                            _ => "key-neutral"
                                        }
                                    )
                                    on:click=move |_| on_key.call(c.to_string())
                                >
                                    {move || {
                                        let id = pulse();
                                        if !id.is_empty() {
                                            if hard() {
                                                view! { <div key=id class="power-ring" /> }.into_view()
                                            } else {
                                                view! { <div key=id class="power-underline" /> }.into_view()
                                            }
                                        } else {
                                            view! {}.into_view()
                                        }
                                    }}
                                    {c.to_string()}
                                </button>
                            }
                        }).collect_view()}

                        {if i == 2 {
                            view! {
                                <button
                                    aria-label="Backspace"
                                    class="key-large px-3 rounded-xl font-black transition-all duration-300 hover:brightness-110 active:scale-90 shadow-lg flex-[1.5] key-neutral flex items-center justify-center"
                                    on:click=move |_| on_key.call("DELETE".to_string())
                                >
                                    <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path d="M12 19l-7-7 7-7m5 14l-7-7 7-7"></path>
                                    </svg>
                                </button>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </div>
                }
            }).collect_view()}
        </footer>
    }
}
