use tauri::State;

use crate::cli::AppPaths;
use crate::view_state::view_state_types::{self, ViewState};

/// Loads the persisted view state from disk.
#[tauri::command]
pub fn load_view_state(state: State<AppPaths>) -> ViewState {
    view_state_types::load_view_state(&state)
}

/// Saves the current view state to disk.
#[tauri::command]
pub fn save_view_state(state: State<AppPaths>, active_sheet_path: Option<String>) {
    view_state_types::save_view_state(&state, &ViewState { active_sheet_path });
}
