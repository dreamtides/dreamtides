use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use parser_v2::ability_directory_parser;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

const DEBOUNCE_DELAY: Duration = Duration::from_secs(5);
const ABILITY_COLUMNS: &[&str] = &["rules-text", "variables"];

#[derive(Debug, Clone, Serialize)]
pub struct AbilityParseStartedPayload {
    pub directory: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AbilityParseCompletedPayload {
    pub directory: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AbilityParseFailedPayload {
    pub directory: String,
    pub error: String,
}

pub struct AbilityParserState {
    inner: Mutex<AbilityParserInner>,
    condvar: Condvar,
    parse_running: AtomicBool,
}

struct AbilityParserInner {
    tabula_directory: Option<PathBuf>,
    parse_pending: bool,
    last_trigger: Option<Instant>,
}

impl AbilityParserState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(AbilityParserInner {
                tabula_directory: None,
                parse_pending: false,
                last_trigger: None,
            }),
            condvar: Condvar::new(),
            parse_running: AtomicBool::new(false),
        }
    }

    pub fn is_ability_column(column_key: &str) -> bool {
        let lower = column_key.to_lowercase();
        ABILITY_COLUMNS.iter().any(|&col| col == lower)
    }

    pub fn set_tabula_directory(&self, path: &Path) {
        if let Some(dir) = path.parent() {
            let mut guard = self.inner.lock().expect("inner lock poisoned");
            if guard.tabula_directory.as_ref() != Some(&dir.to_path_buf()) {
                guard.tabula_directory = Some(dir.to_path_buf());
                tracing::debug!(
                    component = "tv.ability_parser",
                    directory = %dir.display(),
                    "Set tabula directory"
                );
            }
        }
    }

    pub fn trigger_parse(&self) {
        let mut guard = self.inner.lock().expect("inner lock poisoned");
        guard.parse_pending = true;
        guard.last_trigger = Some(Instant::now());
        self.condvar.notify_one();
        tracing::debug!(
            component = "tv.ability_parser",
            "Parse triggered, debounce timer reset"
        );
    }

    pub fn start_background_task(self: &Arc<Self>, app_handle: AppHandle) {
        let state = Arc::clone(self);

        std::thread::spawn(move || {
            tracing::info!(
                component = "tv.ability_parser",
                "Background parser thread started"
            );

            loop {
                {
                    let mut guard = state.inner.lock().expect("inner lock poisoned");
                    while !guard.parse_pending {
                        guard = state.condvar.wait(guard).expect("condvar wait failed");
                    }
                }

                loop {
                    std::thread::sleep(DEBOUNCE_DELAY);

                    let mut guard = state.inner.lock().expect("inner lock poisoned");
                    if let Some(last_trigger) = guard.last_trigger {
                        if last_trigger.elapsed() >= DEBOUNCE_DELAY {
                            guard.parse_pending = false;
                            let dir = guard.tabula_directory.clone();
                            drop(guard);

                            if let Some(dir) = dir {
                                state.run_parser(&app_handle, &dir);
                            }
                            break;
                        }
                    } else {
                        guard.parse_pending = false;
                        break;
                    }
                }
            }
        });
    }

    fn run_parser(&self, app_handle: &AppHandle, tabula_dir: &Path) {
        if self.parse_running.swap(true, Ordering::SeqCst) {
            tracing::debug!(
                component = "tv.ability_parser",
                "Parser already running, skipping"
            );
            return;
        }

        let start = Instant::now();
        let dir_str = tabula_dir.to_string_lossy().to_string();

        tracing::info!(
            component = "tv.ability_parser",
            directory = %dir_str,
            "Starting ability parse"
        );

        let _ = app_handle.emit(
            "ability-parse-started",
            AbilityParseStartedPayload { directory: dir_str.clone() },
        );

        let result = execute_parser(tabula_dir);

        self.parse_running.store(false, Ordering::SeqCst);

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(()) => {
                tracing::info!(
                    component = "tv.ability_parser",
                    directory = %dir_str,
                    duration_ms = duration_ms,
                    "Ability parse completed successfully"
                );
                let _ = app_handle.emit(
                    "ability-parse-completed",
                    AbilityParseCompletedPayload { directory: dir_str, duration_ms },
                );
            }
            Err(error) => {
                tracing::error!(
                    component = "tv.ability_parser",
                    directory = %dir_str,
                    error = %error,
                    duration_ms = duration_ms,
                    "Ability parse failed"
                );
                let _ = app_handle.emit(
                    "ability-parse-failed",
                    AbilityParseFailedPayload { directory: dir_str, error },
                );
            }
        }
    }
}

impl Default for AbilityParserState {
    fn default() -> Self {
        Self::new()
    }
}

fn execute_parser(tabula_dir: &Path) -> Result<(), String> {
    let output_path = tabula_dir.join("parsed_abilities.json");

    tracing::debug!(
        component = "tv.ability_parser",
        tabula_dir = %tabula_dir.display(),
        output_path = %output_path.display(),
        "Executing parser"
    );

    let results = ability_directory_parser::parse_abilities_from_directory(tabula_dir)
        .map_err(|e| format!("Failed to parse abilities: {e}"))?;

    let output_content =
        serde_json::to_string(&results).map_err(|e| format!("Failed to serialize results: {e}"))?;

    fs::write(&output_path, output_content)
        .map_err(|e| format!("Failed to write output file: {e}"))?;

    tracing::debug!(
        component = "tv.ability_parser",
        card_count = results.len(),
        output_path = %output_path.display(),
        "Wrote parsed abilities"
    );

    Ok(())
}
