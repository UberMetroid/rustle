use leptos::*;
use crate::api::post_score;
use crate::state::AppStateContext;

/// Left sidebar containing utility buttons (Stats, Help, Hard Mode, Share).
#[component]
pub fn SidebarLeft(
    /// Callback to start NG+.
    _start_ng_plus: Callback<()>,
    /// Callback to share results.
    share_results: Callback<()>,
) -> impl IntoView {
    let state = use_context::<AppStateContext>().expect("AppStateContext not found");

    view! {
        <aside class="flex flex-col gap-4 shrink-0 absolute left-2 sm:left-4 top-2 scale-75 sm:scale-100 origin-top-left z-40">
            <button aria-label="Leaderboard" on:click=move |_| { state.global_stats_res.refetch(); state.set_show_stats.set(true); } title="Team Status" class="btn-large correct-pad shadow-lg border-2 border-transparent transition-all active:scale-90">
                <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path></svg>
            </button>
            <button aria-label="How to Play" on:click=move |_| state.set_show_help.set(true) title="How to Play" class="btn-large correct-pad shadow-lg border-2 border-transparent transition-all active:scale-90">
                <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
            </button>
            <button aria-label="Toggle Hard Mode" on:click=move |_| {
                if state.is_ng_plus.get() {
                    let msgs = ["NO ESCAPE FROM THE SYSTEM.", "MANDATORY HARD MODE.", "SUCK IT UP, BUTTERCUP.", "YOU ASKED FOR THIS.", "AI DOES NOT ALLOW COWARDICE."];
                    state.set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                    set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                } else if state.guesses.get().is_empty() {
                    state.set_hard_mode.update(|h| *h = !*h);
                    let msgs = if state.hard_mode.get() { ["A GLUTTON FOR PUNISHMENT.", "OH, YOU THINK YOU'RE SMART?", "BRING THE PAIN.", "HARD MODE ENGAGED.", "NO MERCY.", "PREPARE TO SUFFER.", "FINALLY, A CHALLENGE."] } else { ["COWARD.", "TOO HARD FOR YOU?", "BACK TO BABY MODE.", "COPPING OUT ALREADY?", "WEAK AURA.", "I EXPECTED BETTER.", "EASY MODE ENGAGED."] };
                    state.set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                    set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                } else {
                    let msgs = ["TOO LATE TO CHANGE NOW.", "YOU MADE YOUR BED.", "NO BACKING OUT MID-GAME.", "COMMITTED TO THIS PATH.", "NICE TRY."];
                    state.set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                    set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                }
            } title="Hard Mode" class=move || format!("btn-large shadow-lg border-2 transition-all active:scale-90 {}", if state.hard_mode.get() || state.is_ng_plus.get() { "correct-pad border-transparent" } else { "cell-neutral border-current" })>
                <svg class=move || format!("w-6 h-6 sm:w-8 sm:h-8 transition-all {}", if state.hard_mode.get() || state.is_ng_plus.get() { "text-yellow-300 scale-110 drop-shadow-[0_0_12px_rgba(253,224,71,1)]" } else { "text-current opacity-40" }) fill="currentColor" viewBox="0 0 24 24"> <path d="M13 10V3L4 14h7v7l9-11h-7z"></path> </svg>
            </button>
            <button aria-label="Share Results" on:click=move |_| {
                if state.daily_game_done.get() { share_results.call(()); } else {
                    let msgs = ["FINISH THE GAME FIRST.", "EARN IT.", "NOTHING TO SHARE YET.", "NICE TRY."];
                    state.set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                    set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                }
            } title="Share Results" class=move || format!("btn-large shadow-lg border-2 transition-all {}", if state.daily_game_done.get() { "bg-green-500 text-white border-transparent hover:bg-green-400 active:scale-90 shadow-[0_0_15px_rgba(34,197,94,0.8)]" } else { "cell-neutral text-current opacity-40 border-current" })>
                <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"></path></svg>
            </button>
        </aside>
    }
}

/// Right sidebar containing team selection buttons.
#[component]
pub fn SidebarRight() -> impl IntoView {
    let state = use_context::<AppStateContext>().expect("AppStateContext not found");

    view! {
        <aside class="flex flex-col gap-3 shrink-0 absolute right-2 sm:right-4 top-2 scale-75 sm:scale-100 origin-top-right z-40">
            {move || {
                let themes = [
                    ("red", "bg-red-600 text-white", "Red"), ("orange", "bg-orange-500 text-white", "Orange"),
                    ("yellow", "bg-yellow-400 text-black", "Yellow"), ("green", "bg-green-600 text-white", "Green"),
                    ("blue", "bg-blue-600 text-white", "Blue"), ("purple", "bg-purple-600 text-white", "Purple")
                ];
                let cur = state.theme.get();
                let pulse_sig = state.win_pulse_trigger;
                let s = state.global_stats_res.get().unwrap_or_default();
                
                themes.into_iter().map(move |(t, bg, label)| {
                    let t_str = t.to_string();
                    let t_str_2 = t_str.clone();
                    let is_act = cur == t;
                    let winner_val = s.yesterday_winner.clone();
                    
                    view! {
                        <button
                            aria-label=format!("Join {} Team", label)
                            on:click={
                                let t_str = t_str.clone();
                                move |_| {
                                if state.theme.get() != t_str {
                                    state.set_theme.set(t_str.clone());
                                    post_score(t_str.clone(), 0);
                                    let msgs = match t_str.as_str() {
                                        "red" => vec!["SKIBIDI 🚽", "NO CAP 🧢", "GOAT STATUS 🐐", "W RIZZ 🥶"],
                                        "orange" => vec!["VIBE CHECK 💅", "MAIN CHARACTER ENERGY ✨", "TOUCH GRASS 🌿"],
                                        "yellow" => vec!["ADULTING IS HARD ☕", "PUPPERCINO VIBES 🐶"],
                                        "green" => vec!["WHATEVER 🙄", "AS IF ✋"],
                                        "blue" => vec!["BACK IN MY DAY... 👴", "RESPECT YOUR ELDERS 🧐"],
                                        "purple" => vec!["NONSENSE AND TOMFOOLERY 🗞️", "GREAT DEPRESSION SURVIVOR 📻"],
                                        _ => vec!["JOINING TEAM."]
                                    };
                                    state.set_snarky_comment.set(msgs[(js_sys::Math::random() * msgs.len() as f64).floor() as usize].to_string());
                                    set_timeout(move || state.set_snarky_comment.set(String::new()), std::time::Duration::from_millis(6000));
                                }
                            }}
                            title=format!("{} Team", label)
                            class=format!("theme-square {} active:scale-125 relative {}", bg, if is_act { "active ring-2 ring-white ring-offset-2" } else { "" })
                        >
                            <Show when={let t_str_2 = t_str_2.clone(); move || winner_val == t_str_2}>
                                <div class="crown-icon">"👑"</div>
                            </Show>
                            <Show when=move || is_act>
                                <div class="absolute inset-0 flex items-center justify-center font-black drop-shadow-md z-10 text-[10px] sm:text-xs text-white">{state.session_points.get()}</div>
                            </Show>
                            <Show when=move || { let p = pulse_sig.get(); !p.is_empty() && is_act }>
                                <div key=pulse_sig.get() class="win-pulse">{pulse_sig.get()}</div>
                            </Show>
                        </button>
                    }
                }).collect_view()
            }}
        </aside>
    }
}
