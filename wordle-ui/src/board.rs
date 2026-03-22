use leptos::*;
use crate::components::Row;
use crate::state::AppStateContext;

/// The game board grid showing previous guesses, the current input row, and empty rows.
#[component]
pub fn Board() -> impl IntoView {
    let state = use_context::<AppStateContext>().expect("AppStateContext not found");

    view! {
        <div class="flex-1 flex flex-col items-center justify-center min-h-0 py-2 px-10 sm:px-8 relative z-30">
            <div class="flex flex-col gap-1 sm:gap-2 h-full max-h-[480px] aspect-[5/6]">
                {move || {
                    let gs = state.guesses.get();
                    let ss = state.guess_statuses.get();
                    let is_rev = state.is_revealing_row.get();
                    let len = gs.len();
                    let hard = state.hard_mode.get() || state.is_ng_plus.get();
                    
                    gs.into_iter().zip(ss.into_iter()).enumerate().map(move |(i, (g, s))| {
                        view! {
                            <Row
                                guess=g.to_uppercase()
                                statuses=s
                                is_revealing=is_rev && i == len-1
                                is_completed=true
                                is_jiggling=Signal::derive(|| false)
                                destroy_trigger=state.destroy_trigger.into()
                                last_typed_index=-1
                                is_hard_mode=hard
                            />
                        }
                    }).collect_view()
                }}

                {move || if state.guesses.get().len() < 6 && !state.game_won.get() {
                    let input = state.current_input.get().to_uppercase();
                    let last = state.last_typed_index.get();
                    let hard = state.hard_mode.get() || state.is_ng_plus.get();
                    view! {
                        <Row
                            guess=input
                            statuses=vec![]
                            is_revealing=false
                            is_completed=false
                            is_jiggling=Signal::derive(move || state.jiggle_row.get())
                            destroy_trigger=state.destroy_trigger.into()
                            last_typed_index=last
                            is_hard_mode=hard
                        />
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}

                {move || {
                    let hard = state.hard_mode.get() || state.is_ng_plus.get();
                    let empty_count = 6_usize.saturating_sub(
                        state.guesses.get().len() + if state.guesses.get().len() < 6 && !state.game_won.get() { 1 } else { 0 }
                    );
                    (0..empty_count).map(move |_| {
                        view! {
                            <Row
                                guess="".to_string()
                                statuses=vec![]
                                is_revealing=false
                                is_completed=false
                                is_jiggling=Signal::derive(|| false)
                                destroy_trigger=state.destroy_trigger.into()
                                last_typed_index=-1
                                is_hard_mode=hard
                            />
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}
