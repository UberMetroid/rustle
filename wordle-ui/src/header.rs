use leptos::*;

/// The game header containing the title and New Game+ toggle.
#[component]
pub fn Header(
    /// Callback to start NG+.
    start_ng_plus: Callback<()>,
    /// Whether NG+ is active.
    is_ng_plus: ReadSignal<bool>,
    /// Whether daily game is done.
    daily_game_done: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <header class="w-full flex flex-col items-center pt-2 sm:pt-4 shrink-0 relative z-50">
            <div class="flex items-center gap-3">
                <h1 class="text-3xl sm:text-5xl font-black tracking-tighter italic text-center title-text uppercase">"RUSTLE"</h1>
                <button
                    aria-label="Start New Game Plus"
                    on:click=move |_| start_ng_plus.call(())
                    class=move || format!(
                        "w-8 h-8 sm:w-10 sm:h-10 flex items-center justify-center rounded-xl shadow-lg border-2 transition-all active:scale-90 {}",
                        if is_ng_plus.get() || daily_game_done.get() { "ai-active-pad border-transparent shadow-[0_0_15px_rgba(255,0,255,0.8)]" } else { "cell-neutral border-current opacity-40 hover:opacity-100" }
                    )
                >
                    <span class=move || format!("text-xl sm:text-2xl font-black {}", if is_ng_plus.get() || daily_game_done.get() { "text-white" } else { "text-current opacity-50" })>"+"</span>
                </button>
            </div>
        </header>
    }
}

/// The snarky toast message display.
#[component]
pub fn SnarkyToast(
    snarky_comment: ReadSignal<String>,
    game_won: ReadSignal<bool>,
    game_lost: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div aria-live="polite" aria-atomic="true" class="flex-1 flex items-end justify-center pointer-events-none px-4 relative z-50 pb-2">
            {move || {
                let snark = snarky_comment.get();
                if !snark.is_empty() {
                    let color = if game_won.get() { "text-green-400" } else if game_lost.get() { "text-red-400" } else { "text-theme-primary" };
                    view! {
                        <div key=snark.clone() class=format!("text-[10px] sm:text-xs font-black uppercase tracking-widest {} snarky-toast text-center pointer-events-auto", color)>
                            {snark}
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}
        </div>
    }
}
