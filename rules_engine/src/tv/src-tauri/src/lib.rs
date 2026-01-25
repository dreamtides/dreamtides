use std::collections::HashSet;

use tauri::Manager;

pub mod cli;
#[path = "commands/commands_mod.rs"]
mod commands;
#[path = "derived/derived_mod.rs"]
pub mod derived;
#[path = "error/error_mod.rs"]
pub mod error;
#[path = "images/images_mod.rs"]
mod images;
#[path = "logging/logging_mod.rs"]
mod logging;
#[path = "sync/sync_mod.rs"]
mod sync;
#[path = "toml/toml_mod.rs"]
pub mod toml;
#[path = "traits/traits_mod.rs"]
pub mod traits;
#[path = "uuid/uuid_mod.rs"]
mod uuid;
#[path = "validation/validation_mod.rs"]
mod validation;

#[tauri::command]
fn get_app_paths(state: tauri::State<cli::AppPaths>) -> Vec<String> {
    state.files.iter().map(|p| p.to_string_lossy().to_string()).collect()
}

fn cleanup_temp_files_on_startup(paths: &cli::AppPaths) {
    let mut directories = HashSet::new();
    for file in &paths.files {
        if let Some(parent) = file.parent() {
            directories.insert(parent.to_path_buf());
        }
    }

    for dir in directories {
        if let Err(e) = toml::document_writer::cleanup_orphaned_temp_files(&dir.to_string_lossy()) {
            tracing::warn!(
                component = "tv.toml",
                dir = %dir.display(),
                error = %e,
                "Failed to clean up orphaned temp files"
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(paths: cli::AppPaths, _jsonl: bool) {
    derived::fluent_integration::initialize_fluent_resource();
    derived::function_registry::initialize_global_registry();
    cleanup_temp_files_on_startup(&paths);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(paths)
        .manage(sync::file_watcher::FileWatcherState::new())
        .manage(sync::state_machine::SyncStateMachineState::new())
        .invoke_handler(tauri::generate_handler![
            commands::load_command::load_toml_table,
            commands::save_command::save_toml_table,
            commands::save_command::save_cell,
            commands::save_command::save_batch,
            commands::watch_command::start_file_watcher,
            commands::watch_command::stop_file_watcher,
            get_app_paths,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                sync::file_watcher::stop_all_watchers(window.app_handle());
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
