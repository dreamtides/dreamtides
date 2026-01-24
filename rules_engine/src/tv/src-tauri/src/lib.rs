pub mod cli;
mod file_watcher;
mod toml_loader;

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
            file_watcher::start_file_watcher,
            toml_loader::load_toml_table,
            toml_loader::save_toml_table,
            get_app_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
