mod file_watcher;
mod toml_loader;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            file_watcher::start_file_watcher,
            toml_loader::load_toml_table,
            toml_loader::save_toml_table,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
