use leptos::*;

/// A single letter cell in the Wordle grid.
#[component]
pub fn Cell(
    value: char,
    status: String,
    is_revealing: bool,
    reveal_delay: usize,
    is_completed: bool,
    destroy_trigger: Signal<String>,
    index: usize,
    last_typed_index: i32,
    is_hard_mode: bool,
) -> impl IntoView {
    let status_c = status.clone();
    let is_last = last_typed_index == index as i32;

    let class = {
        let status_c = status_c.clone();
        move || {
            let mut base = "cell text-2xl sm:text-3xl font-black flex items-center justify-center rounded-lg sm:rounded-xl border-2 transition-all duration-500 uppercase select-none".to_string();
            if is_revealing { base.push_str(" revealing"); }
            if is_completed || is_revealing {
                if !status_c.is_empty() { base.push_str(&format!(" {}", status_c)); } else { base.push_str(" cell-neutral"); }
            } else if value != ' ' {
                base.push_str(" border-theme-primary text-theme-primary");
                if is_last { base.push_str(" pop-animation"); }
            } else {
                base.push_str(" border-gray-700 text-gray-500 opacity-20");
            }
            base
        }
    };

    let aria_label = move || {
        if value == ' ' {
            "Empty cell".to_string()
        } else if !status_c.is_empty() {
            format!("{} {}", value, status_c)
        } else {
            value.to_string()
        }
    };

    view! {
        <div class=class style=format!("transition-delay: {}ms", reveal_delay) aria-label=aria_label>
            <Show when=move || !destroy_trigger.get().is_empty() && is_last>
                <div key=destroy_trigger.get() class="destroy-particle" />
            </Show>
            {if value == ' ' { "".to_string() } else { value.to_string() }}
            <Show when={let status_c = status_c.clone(); move || is_revealing && status_c == "correct"}>
                {if is_hard_mode { view! { <div class="power-ring" /> }.into_view() } else { view! { <div class="power-underline" /> }.into_view() }}
            </Show>
        </div>
    }
}

/// A row of 5 cells representing a single guess.
#[component]
pub fn Row(
    guess: String,
    statuses: Vec<String>,
    is_revealing: bool,
    is_completed: bool,
    is_jiggling: Signal<bool>,
    destroy_trigger: Signal<String>,
    last_typed_index: i32,
    is_hard_mode: bool,
) -> impl IntoView {
    let chars: Vec<char> = guess.chars().chain(std::iter::repeat(' ')).take(5).collect();
    let row_aria_label = move || {
        if guess.is_empty() {
            "Empty guess row".to_string()
        } else {
            format!("Guess: {}", guess)
        }
    };

    view! {
        <div class=move || format!("grid grid-cols-5 gap-1 sm:gap-2 w-full h-full {}", if is_jiggling.get() { "jiggle" } else { "" }) role="row" aria-label=row_aria_label>
            {chars.into_iter().enumerate().map(|(i, c)| {
                let status = statuses.get(i).cloned().unwrap_or_default();
                view! { <Cell value=c status=status is_revealing reveal_delay=i * 300 is_completed destroy_trigger index=i last_typed_index is_hard_mode /> }
            }).collect_view()}
        </div>
    }
}

/// A stylized modal popup for stats and help.
#[component]
pub fn Modal(
    title: String,
    is_open: ReadSignal<bool>,
    set_is_open: WriteSignal<bool>,
    children: Children,
) -> impl IntoView {
    let children = children(); // Call ONCE
    view! {
        <Show when=move || is_open.get()>
            <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6 bg-black bg-opacity-80 backdrop-blur-md animate-fade-in">
                <div class="relative w-full max-w-lg bg-theme-bg border-2 border-theme-primary rounded-2xl sm:rounded-3xl shadow-[0_0_50px_rgba(0,0,0,0.5)] overflow-hidden max-h-[90vh] flex flex-col scale-in" on:click=move |e| e.stop_propagation()>
                    <div class="flex items-center justify-between p-4 sm:p-6 border-b border-white border-opacity-10 shrink-0">
                        <h2 class="text-xl sm:text-2xl font-black italic tracking-tighter uppercase title-text"> {title.clone()} </h2>
                        <button on:click=move |_| set_is_open.set(false) class="p-2 hover:bg-white hover:bg-opacity-10 rounded-full transition-colors">
                            <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                        </button>
                    </div>
                    <div class="flex-1 overflow-y-auto p-4 sm:p-8 custom-scrollbar">
                        {children.clone()}
                    </div>
                </div>
            </div>
        </Show>
    }
}
