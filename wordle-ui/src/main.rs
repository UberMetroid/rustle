//! RUSTLE Frontend Entry Point
//! 
//! This module initializes the Leptos application and mounts it to the DOM.
//! It also handles global module definitions and JS bindings for fireworks.

use leptos::*;
use wasm_bindgen::prelude::*;

/// API interaction and data structures for server communication.
pub mod api;
/// Themed commentary and emoji logic for team flair.
pub mod comments;
/// Reusable UI layout components (Cells, Rows, Modals).
pub mod components;
/// The core App component orchestration.
pub mod app;
/// On-screen virtual keyboard with status highlighting.
pub mod keyboard;
/// The main game grid board.
pub mod board;
/// Left and right utility sidebars.
pub mod sidebar;
/// Top title bar and New Game+ controls.
pub mod header;
/// Leaderboard and instruction modal popups.
pub mod modals;
/// Centralized reactive state context.
pub mod state;
/// Shared game logic helpers (Sharing, Storage).
pub mod app_helpers;
/// The main keyboard event handler and game state machine.
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
    /// JS-native function to trigger cyberpunk fireworks canvas.
    #[wasm_bindgen(js_name = cyberpunkVictory)]
    fn celebrate(theme: &str, is_hard: bool, is_ng_plus: bool);
}

/// Main entry point for the WASM application.
fn main() {
    // Catch panics and log them to the browser console.
    console_error_panic_hook::set_once();
    
    // Mount the Leptos app to the #root element.
    let root = document()
        .get_element_by_id("root")
        .expect("could not find #root element");
    mount_to(root.unchecked_into(), || view! { <App/> })
}

