pub mod cli;
#[path = "commands/commands_mod.rs"]
mod commands;
#[path = "derived/derived_mod.rs"]
mod derived;
#[path = "error/error_mod.rs"]
mod error;
#[path = "images/images_mod.rs"]
mod images;
#[path = "logging/logging_mod.rs"]
mod logging;
#[path = "sync/sync_mod.rs"]
mod sync;
#[path = "toml/toml_mod.rs"]
mod toml;
#[path = "uuid/uuid_mod.rs"]
mod uuid;
#[path = "validation/validation_mod.rs"]
mod validation;

#[tauri::command]
fn get_app_paths(state: tauri::State<cli::AppPaths>) -> Vec<String> {
    state
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(paths: cli::AppPaths, _jsonl: bool) {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(paths)
        .invoke_handler(tauri::generate_handler![
            commands::load_command::load_toml_table,
            commands::save_command::save_toml_table,
            commands::watch_command::start_file_watcher,
            get_app_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
