use tauri::AppHandle;

use crate::sync::file_watcher;

/// Tauri command to start a file watcher for the given file path.
#[tauri::command]
pub fn start_file_watcher(app_handle: AppHandle, file_path: String) {
    file_watcher::start_watcher(app_handle, file_path);
}
