use std::collections::HashSet;

use tauri::Manager;

use crate::derived::compute_executor::ComputeExecutorState;
use crate::filter::filter_state::FilterStateManager;
use crate::images::image_fetcher::ImageFetcherState;

pub mod cli;
#[path = "commands/commands_mod.rs"]
mod commands;
#[path = "derived/derived_mod.rs"]
pub mod derived;
#[path = "error/error_mod.rs"]
pub mod error;
#[path = "filter/filter_mod.rs"]
pub mod filter;
#[path = "images/images_mod.rs"]
pub mod images;
#[path = "logging/logging_mod.rs"]
pub mod logging;
#[path = "sort/sort_mod.rs"]
pub mod sort;
#[path = "sync/sync_mod.rs"]
mod sync;
#[path = "toml/toml_mod.rs"]
pub mod toml;
#[path = "traits/traits_mod.rs"]
pub mod traits;
#[path = "uuid/uuid_mod.rs"]
mod uuid;
#[path = "validation/validation_mod.rs"]
pub mod validation;
#[path = "view_state/view_state_mod.rs"]
pub mod view_state;

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

fn initialize_compute_executor() -> ComputeExecutorState {
    let state = ComputeExecutorState::new();
    if let Err(e) = state.initialize(None) {
        tracing::error!(
            component = "tv.derived.executor",
            error = %e,
            "Failed to initialize compute executor"
        );
    }
    state
}

fn initialize_image_fetcher(app_handle: &tauri::AppHandle) {
    if let Some(state) = app_handle.try_state::<ImageFetcherState>() {
        let cache_dir = app_handle
            .path()
            .app_cache_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from(".tv_cache"));
        if let Err(e) = state.initialize(&cache_dir) {
            tracing::error!(
                component = "tv.images.fetcher",
                error = %e,
                "Failed to initialize image fetcher"
            );
        }
    }
}

fn stop_compute_executor(app_handle: &tauri::AppHandle) {
    if let Some(state) = app_handle.try_state::<ComputeExecutorState>() {
        state.stop();
        tracing::info!(
            component = "tv.derived.executor",
            "Compute executor stopped"
        );
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(paths: cli::AppPaths) {
    logging::json_logger::initialize();
    derived::fluent_integration::initialize_fluent_resource();
    derived::function_registry::initialize_global_registry();
    cleanup_temp_files_on_startup(&paths);

    let executor_state = initialize_compute_executor();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(paths)
        .manage(sync::file_watcher::FileWatcherState::new())
        .manage(sync::state_machine::SyncStateMachineState::new())
        .manage(sort::sort_state::SortStateManager::new())
        .manage(FilterStateManager::new())
        .manage(executor_state)
        .manage(ImageFetcherState::new())
        .invoke_handler(tauri::generate_handler![
            commands::load_command::load_toml_table,
            commands::save_command::save_toml_table,
            commands::save_command::save_cell,
            commands::save_command::save_batch,
            commands::save_command::add_row,
            commands::sort_command::get_sort_state,
            commands::sort_command::set_sort_state,
            commands::sort_command::clear_sort_state,
            commands::sort_command::get_sort_row_mapping,
            commands::sort_command::translate_row_index,
            commands::watch_command::start_file_watcher,
            commands::watch_command::stop_file_watcher,
            commands::derived_command::compute_derived,
            commands::derived_command::compute_derived_batch,
            commands::derived_command::update_lookup_context,
            commands::derived_command::increment_row_generation,
            commands::derived_command::clear_computation_queue,
            commands::derived_command::get_computation_queue_length,
            commands::derived_command::get_derived_columns_config,
            commands::filter_command::get_filter_state,
            commands::filter_command::set_filter_state,
            commands::filter_command::clear_filter_state,
            commands::filter_command::get_filter_visibility,
            commands::filter_command::is_row_visible,
            commands::filter_command::set_hidden_rows,
            commands::filter_command::get_hidden_rows,
            commands::validation_command::get_validation_rules,
            commands::validation_command::get_enum_validation_rules,
            commands::style_command::get_table_style,
            commands::style_command::get_available_color_schemes,
            commands::style_command::get_conditional_formatting,
            commands::image_command::fetch_image,
            commands::row_command::get_row_config,
            commands::column_command::get_column_configs,
            commands::column_command::set_column_width,
            commands::column_command::set_derived_column_width,
            commands::log_command::log_message,
            commands::view_state_command::load_view_state,
            commands::view_state_command::save_view_state,
            get_app_paths,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Start the compute executor after setup
            if let Some(state) = app_handle.try_state::<ComputeExecutorState>() {
                state.start(app_handle.clone());
                tracing::info!(
                    component = "tv.derived.executor",
                    "Compute executor started"
                );
            }

            // Initialize the image fetcher with the app cache directory
            initialize_image_fetcher(&app_handle);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                sync::file_watcher::stop_all_watchers(window.app_handle());
                stop_compute_executor(window.app_handle());
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
