use tauri::State;

use crate::cli::AppPaths;
use crate::view_state::sheet_order;

/// Loads the persisted sheet tab order from `sheets.toml`.
#[tauri::command]
pub fn load_sheet_order(state: State<AppPaths>) -> sheet_order::SheetOrder {
    sheet_order::load_sheet_order(&state)
}

/// Saves the sheet tab order to `sheets.toml`.
#[tauri::command]
pub fn save_sheet_order(state: State<AppPaths>, order: Vec<String>) {
    sheet_order::save_sheet_order(&state, &sheet_order::SheetOrder { order });
}
