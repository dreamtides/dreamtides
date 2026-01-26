use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cli::AppPaths;

const VIEW_STATE_FILENAME: &str = ".tv_view_state.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewState {
    pub active_sheet_path: Option<String>,
}

/// Derives the `.tv_view_state.json` path from the parent directory of the
/// first file in AppPaths.
pub fn view_state_path(app_paths: &AppPaths) -> Option<PathBuf> {
    app_paths.files.first()?.parent().map(|p| p.join(VIEW_STATE_FILENAME))
}

/// Reads and parses the view state JSON file. Returns default on any error.
pub fn load_view_state(app_paths: &AppPaths) -> ViewState {
    let Some(path) = view_state_path(app_paths) else {
        return ViewState::default();
    };
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return ViewState::default();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

/// Writes the view state JSON file. Logs a warning on error but never fails.
pub fn save_view_state(app_paths: &AppPaths, state: &ViewState) {
    let Some(path) = view_state_path(app_paths) else {
        tracing::warn!(
            component = "tv.view_state",
            "Cannot determine view state path: no files in AppPaths"
        );
        return;
    };
    match serde_json::to_string_pretty(state) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                tracing::warn!(
                    component = "tv.view_state",
                    path = %path.display(),
                    error = %e,
                    "Failed to write view state file"
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                component = "tv.view_state",
                error = %e,
                "Failed to serialize view state"
            );
        }
    }
}
