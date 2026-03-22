use leptos::*;

/// A stylized modal popup for stats and help.
#[component]
pub fn Modal(
    /// Title displayed at the top.
    title: String,
    /// Reactive signal controlling visibility.
    is_open: ReadSignal<bool>,
    /// Setter to close the modal.
    set_is_open: WriteSignal<bool>,
    /// The content to display inside the modal.
    children: Children,
) -> impl IntoView {
    view! {
        <Show when=move || is_open.get()>
            <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6 bg-black bg-opacity-80 backdrop-blur-md animate-fade-in">
                <div class="relative w-full max-w-lg bg-theme-bg border-2 border-theme-primary rounded-2xl sm:rounded-3xl shadow-[0_0_50px_rgba(0,0,0,0.5)] overflow-hidden max-h-[90vh] flex flex-col scale-in" on:click=move |e| e.stop_propagation()>
                    <div class="flex items-center justify-between p-4 sm:p-6 border-b border-white border-opacity-10 shrink-0">
                        <h2 class="text-xl sm:text-2xl font-black italic tracking-tighter uppercase title-text"> {title.clone()} </h2>
                        <button on:click=move |_| set_is_open.set(false) class="p-2 hover:bg-white hover:bg-opacity-10 rounded-full transition-colors">
                            <svg class="w-6 h-6 sm:w-8 sm:h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                    <div class="flex-1 overflow-y-auto p-4 sm:p-8 custom-scrollbar">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}
