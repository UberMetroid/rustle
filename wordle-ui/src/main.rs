use leptos::*;
use wasm_bindgen::prelude::*;

pub mod api;
pub mod comments;
pub mod components;
pub mod app;
pub mod keyboard;
pub mod board;
pub mod sidebar;
pub mod header;
pub mod modals;
pub mod state;
pub mod app_helpers;
pub mod app_on_key;

pub use api::*;
pub use comments::*;
pub use components::*;
pub use app::*;
pub use keyboard::*;
pub use board::*;
pub use sidebar::*;
pub use header::*;
pub use modals::*;
pub use state::*;
pub use app_helpers::*;
pub use app_on_key::*;

use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = cyberpunkVictory)]
    fn celebrate(theme: &str, is_hard: bool, is_ng_plus: bool);
}

fn main() {
    console_error_panic_hook::set_once();
    let root = document()
        .get_element_by_id("root")
        .expect("could not find #root element");
    mount_to(root.unchecked_into(), || view! { <App/> })
}
