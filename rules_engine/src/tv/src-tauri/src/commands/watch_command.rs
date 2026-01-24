use tauri::AppHandle;

use crate::error::error_types::TvError;
use crate::sync::file_watcher;

/// Tauri command to start a file watcher for the given file path.
#[tauri::command]
pub fn start_file_watcher(app_handle: AppHandle, file_path: String) -> Result<(), TvError> {
    file_watcher::start_watcher(app_handle, file_path)
}

/// Tauri command to stop a file watcher for the given file path.
#[tauri::command]
pub fn stop_file_watcher(app_handle: AppHandle, file_path: String) -> Result<(), TvError> {
    file_watcher::stop_watcher(app_handle, file_path)
}
