use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cli::AppPaths;

const SHEETS_TOML_FILENAME: &str = "sheets.toml";

/// Persisted sheet ordering metadata.
///
/// Stored as a `sheets.toml` file in the same directory as the TOML data files.
/// Contains the ordered list of sheet filenames that determines tab order in the
/// spreadsheet UI. When a user drags sheet tabs to reorder them, the new order
/// is persisted here and used on subsequent launches.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SheetOrder {
    /// Ordered list of TOML filenames (e.g. `["cards.toml", "items.toml"]`).
    /// Files not listed here are appended alphabetically after the listed ones.
    pub order: Vec<String>,
}

/// Derives the `sheets.toml` path from the parent directory of the first file
/// in AppPaths.
pub fn sheet_order_path(app_paths: &AppPaths) -> Option<PathBuf> {
    app_paths.files.first()?.parent().map(|p| p.join(SHEETS_TOML_FILENAME))
}

/// Reads and parses the sheet order from `sheets.toml`. Returns default on any
/// error (missing file, parse error, etc).
pub fn load_sheet_order(app_paths: &AppPaths) -> SheetOrder {
    let Some(path) = sheet_order_path(app_paths) else {
        return SheetOrder::default();
    };
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return SheetOrder::default();
    };
    toml::from_str(&contents).unwrap_or_default()
}

/// Writes the sheet order to `sheets.toml`. Logs a warning on error but never
/// fails.
pub fn save_sheet_order(app_paths: &AppPaths, sheet_order: &SheetOrder) {
    let Some(path) = sheet_order_path(app_paths) else {
        tracing::warn!(
            component = "tv.view_state",
            "Cannot determine sheet order path: no files in AppPaths"
        );
        return;
    };
    match toml::to_string_pretty(sheet_order) {
        Ok(toml_str) => {
            if let Err(e) = std::fs::write(&path, toml_str) {
                tracing::warn!(
                    component = "tv.view_state",
                    path = %path.display(),
                    error = %e,
                    "Failed to write sheet order file"
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                component = "tv.view_state",
                error = %e,
                "Failed to serialize sheet order"
            );
        }
    }
}
