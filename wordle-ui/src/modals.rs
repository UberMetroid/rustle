use leptos::*;
use crate::api::{GlobalStats, GameStats};
use crate::components::Modal;

/// Modals for Leaderboard and How to Play.
#[component]
pub fn GameModals(
    show_stats: ReadSignal<bool>,
    set_show_stats: WriteSignal<bool>,
    show_help: ReadSignal<bool>,
    set_show_help: WriteSignal<bool>,
    global_stats_res: Resource<(), GlobalStats>,
    stats: ReadSignal<GameStats>,
) -> impl IntoView {
    view! {
        <Modal title="Leaderboard".to_string() is_open=show_stats set_is_open=set_show_stats>
            <div class="flex flex-col items-center text-center text-white p-2 rounded-xl bg-black bg-opacity-40">
                <div class="w-full mb-6 text-center">
                    <h3 class="text-xs font-bold uppercase mb-4 tracking-widest text-theme-primary">"TEAMS"</h3>
                    <div class="grid grid-cols-3 sm:grid-cols-6 w-full gap-2 px-2">
                        {move || {
                            let gs = global_stats_res.get().unwrap_or_default();
                            let team_data = [
                                ("red", "bg-red-600 text-white", "R"),
                                ("orange", "bg-orange-500 text-white", "O"),
                                ("yellow", "bg-yellow-400 text-black", "Y"),
                                ("green", "bg-green-600 text-white", "G"),
                                ("blue", "bg-blue-600 text-white", "B"),
                                ("purple", "bg-purple-600 text-white", "P")
                            ];
                            team_data.into_iter().map(move |(t, bg, _)| {
                                let d = match t {
                                    "yellow" => &gs.yellow,
                                    "red" => &gs.red,
                                    "green" => &gs.green,
                                    "blue" => &gs.blue,
                                    "purple" => &gs.purple,
                                    _ => &gs.orange
                                };
                                let win_name = gs.yesterday_winner.clone();
                                view! {
                                    <div class="flex flex-col items-center">
                                        <div class=format!("theme-square {} active ring-2 ring-white ring-opacity-20", bg)>
                                            <Show when=move || win_name == t>
                                                <div class="crown-icon">"👑"</div>
                                            </Show>
                                            <div class="text-[10px] font-black">{d.points}</div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>
                <div class="w-full border-t border-white border-opacity-10 pt-6 mb-6">
                    <h3 class="text-xs font-bold uppercase mb-4 tracking-widest text-theme-primary text-center">"Individual Ranking"</h3>
                    <div class="grid grid-cols-4 w-full gap-4 mb-8">
                        <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().total_games}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Played"</div></div>
                        <div class="flex flex-col"><div class="text-3xl font-black">{move || if stats.get().total_games > 0 { stats.get().wins * 100 / stats.get().total_games } else { 0 }}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Win %"</div></div>
                        <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().current_streak}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Streak"</div></div>
                        <div class="flex flex-col"><div class="text-3xl font-black">{move || stats.get().best_streak}</div><div class="text-[8px] uppercase opacity-70 tracking-tighter">"Max Streak"</div></div>
                    </div>
                </div>
            </div>
        </Modal>

        <Modal title="How to Play".to_string() is_open=show_help set_is_open=set_show_help>
            <div class="flex flex-col gap-6 text-white text-center">
                <div class="space-y-4">
                    <div class="flex flex-col items-center gap-1">
                        <div class="flex">
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent correct font-black">"R"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"U"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"T"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                        </div>
                        <div class="text-[10px] opacity-70">"R is in the correct spot."</div>
                    </div>
                    <div class="flex flex-col items-center gap-1">
                        <div class="flex">
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral font-bold">"W"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent present mx-0.5 font-black text-black">"O"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"R"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"D"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"S"</div>
                        </div>
                        <div class="text-[10px] opacity-70">"O is in the word but wrong spot."</div>
                    </div>
                    <div class="flex flex-col items-center gap-1">
                        <div class="flex">
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral font-bold">"V"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"A"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"G"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent absent mx-0.5 font-black">"U"</div>
                            <div class="w-10 h-10 flex items-center justify-center rounded-lg border-2 border-transparent cell-neutral mx-0.5 font-bold">"E"</div>
                        </div>
                        <div class="text-[10px] opacity-70">"U is not in the word."</div>
                    </div>
                </div>
                <div class="w-full border-t border-white border-opacity-10 pt-4 flex flex-col gap-4">
                    <div>
                        <h3 class="text-xs font-bold uppercase mb-2 tracking-widest text-theme-primary">"Hard Mode ⚡"</h3>
                        <p class="text-[10px] opacity-80 leading-relaxed">"Any revealed hints must be used in subsequent guesses."</p>
                    </div>
                    <div>
                        <h3 class="text-xs font-bold uppercase mb-2 tracking-widest text-theme-primary">"New Game+"</h3>
                        <p class="text-[10px] opacity-80 leading-relaxed">"In this mode, the AI dynamically dodges your guesses by switching the answer."</p>
                    </div>
                </div>
            </div>
        </Modal>
    }
}
