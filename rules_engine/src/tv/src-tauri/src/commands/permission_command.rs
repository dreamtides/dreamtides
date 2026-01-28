use tauri::AppHandle;

use crate::error::error_types::TvError;
use crate::error::permission_recovery::{
    self, PermissionState, PermissionStateChangedPayload,
};

/// Tauri command to get the current permission state for a file.
#[tauri::command]
pub fn get_permission_state(app_handle: AppHandle, file_path: String) -> PermissionState {
    permission_recovery::get_permission_state(&app_handle, &file_path)
}

/// Tauri command to get the number of pending updates for a file.
#[tauri::command]
pub fn get_pending_update_count(app_handle: AppHandle, file_path: String) -> usize {
    permission_recovery::get_pending_update_count(&app_handle, &file_path)
}

/// Tauri command to retry pending updates after permissions are restored.
#[tauri::command]
pub fn retry_pending_updates(
    app_handle: AppHandle,
    file_path: String,
) -> Result<usize, TvError> {
    // First, check if permissions have been restored
    let current_state = permission_recovery::detect_permission_state(std::path::Path::new(&file_path));

    if current_state != PermissionState::ReadWrite {
        let message = permission_recovery::get_permission_error_message(current_state, &file_path);
        permission_recovery::set_permission_state(&app_handle, &file_path, current_state, &message);
        return Err(TvError::PermissionDenied {
            path: file_path,
            operation: if current_state == PermissionState::Unreadable {
                "read".to_string()
            } else {
                "write".to_string()
            },
        });
    }

    // Permissions restored, try to apply pending updates
    let success_count = permission_recovery::retry_pending_updates(&app_handle, &file_path)?;

    // If all updates succeeded, set state back to ReadWrite
    let remaining = permission_recovery::get_pending_update_count(&app_handle, &file_path);
    if remaining == 0 {
        permission_recovery::set_permission_state(
            &app_handle,
            &file_path,
            PermissionState::ReadWrite,
            "Permissions restored, all pending changes applied.",
        );
    }

    Ok(success_count)
}

/// Tauri command to check and update permission state for a file.
#[tauri::command]
pub fn check_permission_state(
    app_handle: AppHandle,
    file_path: String,
) -> PermissionStateChangedPayload {
    let current_state = permission_recovery::detect_permission_state(std::path::Path::new(&file_path));
    let message = permission_recovery::get_permission_error_message(current_state, &file_path);
    permission_recovery::set_permission_state(&app_handle, &file_path, current_state, &message);

    let pending_count = permission_recovery::get_pending_update_count(&app_handle, &file_path);

    PermissionStateChangedPayload {
        file_path,
        state: current_state,
        message,
        pending_update_count: pending_count,
    }
}

/// Tauri command to clear permission state when a file is no longer being watched.
#[tauri::command]
pub fn clear_permission_state(app_handle: AppHandle, file_path: String) {
    permission_recovery::clear_permission_state(&app_handle, &file_path);
}
