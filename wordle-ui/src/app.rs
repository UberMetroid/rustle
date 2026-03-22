use leptos::*;
use std::collections::HashMap;
use crate::*;
use wordle_engine::*;

/// The root component of the RUSTLE application.
#[component]
pub fn App() -> impl IntoView {
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
        let ts = match global_stats_res.get() { Some(s) => s.server_utc_timestamp, None => js_sys::Date::now() as u64 };
        let val = get_solution(ts);
        serde_wasm_bindgen::from_value::<SolutionData>(val).unwrap_or_else(|_| SolutionData { solution: "APPLE".to_string(), solution_game_date: 0, solution_index: 0, tomorrow: 0 })
    });

    let char_statuses = create_memo(move |_| {
        let mut map = HashMap::new();
        let (gs, ss) = (guesses.get(), guess_statuses.get());
        for (g, s_row) in gs.iter().zip(ss.iter()) {
            for (c, s) in g.chars().zip(s_row.iter()) {
                let current = map.entry(c).or_insert_with(|| s.clone());
                if s == "correct" || (s == "present" && *current != "correct") || (s == "absent" && *current != "correct" && *current != "present") { *current = s.clone(); }
            }
        }
        map
    });

    let context = AppStateContext {
        guesses, set_guesses, guess_statuses, set_guess_statuses_vec, current_input, set_current_input,
        game_won, set_game_won, game_lost, set_game_lost, show_stats, set_show_stats, show_help, set_show_help,
        jiggle_row, set_jiggle_row, is_revealing_row, set_is_revealing_row, destroy_trigger, set_destroy_trigger,
        last_typed_index, set_last_typed_index, theme, set_theme, hard_mode, set_hard_mode, stats, set_stats,
        keyboard_pulse, set_keyboard_pulse, snarky_comment, set_snarky_comment, is_ng_plus, set_is_ng_plus,
        ai_pool, set_ai_pool, daily_game_done, set_daily_game_done, win_pulse_trigger, set_win_pulse_trigger,
        session_points, set_session_points, point_locked_team, set_point_locked_team, global_stats_res,
        solution_data, char_statuses,
    };
    provide_context(context);

    // Create handlers as StoredValue to avoid move issues
    let on_key_sv = store_value(get_on_key(context));
    let share_results_sv = store_value(get_share_results(context));
    let start_ng_plus_sv = store_value(get_start_ng_plus(context));

    let _ = window_event_listener(leptos::ev::keydown, move |ev| {
        if show_stats.get() { if ev.key() == "Enter" && (game_won.get() || game_lost.get()) { share_results_sv.with_value(|f| f()); } return; }
        if show_help.get() { return; }
        match ev.key().as_str() {
            "Enter" => on_key_sv.with_value(|f| f("ENTER".to_string())),
            "Backspace" => on_key_sv.with_value(|f| f("DELETE".to_string())),
            k if k.len() == 1 && k.chars().next().unwrap().is_ascii_alphabetic() => on_key_sv.with_value(|f| f(k.to_uppercase())),
            _ => {}
        }
    });

    view! {
        <div class="flex flex-col h-full bg-app-bg text-app-text overflow-hidden transition-all duration-500 relative">
            <Header start_ng_plus=Callback::new(move |_| start_ng_plus_sv.with_value(|f| f())) is_ng_plus daily_game_done />
            <main class="flex-1 flex flex-col w-full max-w-2xl mx-auto px-2 sm:px-4 min-h-0 relative">
                <SnarkyToast snarky_comment game_won game_lost />
                <div class="w-full flex justify-center relative">
                    <SidebarLeft share_results=Callback::new(move |_| share_results_sv.with_value(|f| f())) _start_ng_plus=Callback::new(move |_| start_ng_plus_sv.with_value(|f| f())) />
                    <Board />
                    <SidebarRight />
                </div>
            </main>
            <Keyboard on_key=Callback::new(move |k| on_key_sv.with_value(|f| f(k))) />
            <GameModals show_stats set_show_stats show_help set_show_help global_stats_res stats />
        </div>
    }
}
