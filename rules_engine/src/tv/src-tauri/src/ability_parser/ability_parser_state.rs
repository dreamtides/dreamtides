use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use tokio::sync::Notify;

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
    tabula_directory: Mutex<Option<PathBuf>>,
    parse_pending: AtomicBool,
    parse_running: AtomicBool,
    notify: Arc<Notify>,
    last_trigger: Mutex<Option<Instant>>,
}

impl AbilityParserState {
    pub fn new() -> Self {
        Self {
            tabula_directory: Mutex::new(None),
            parse_pending: AtomicBool::new(false),
            parse_running: AtomicBool::new(false),
            notify: Arc::new(Notify::new()),
            last_trigger: Mutex::new(None),
        }
    }

    pub fn is_ability_column(column_key: &str) -> bool {
        let lower = column_key.to_lowercase();
        ABILITY_COLUMNS.iter().any(|&col| col == lower)
    }

    pub fn set_tabula_directory(&self, path: &Path) {
        if let Some(dir) = path.parent() {
            let mut guard = self.tabula_directory.lock().expect("tabula_directory lock poisoned");
            if guard.as_ref() != Some(&dir.to_path_buf()) {
                *guard = Some(dir.to_path_buf());
                tracing::debug!(
                    component = "tv.ability_parser",
                    directory = %dir.display(),
                    "Set tabula directory"
                );
            }
        }
    }

    pub fn trigger_parse(&self) {
        self.parse_pending.store(true, Ordering::SeqCst);
        *self.last_trigger.lock().expect("last_trigger lock poisoned") = Some(Instant::now());
        self.notify.notify_one();
        tracing::debug!(
            component = "tv.ability_parser",
            "Parse triggered, debounce timer reset"
        );
    }

    pub fn is_parse_running(&self) -> bool {
        self.parse_running.load(Ordering::SeqCst)
    }

    pub fn start_background_task(self: Arc<Self>, app_handle: AppHandle) {
        let state = Arc::clone(&self);
        let notify = Arc::clone(&self.notify);

        tokio::spawn(async move {
            tracing::info!(
                component = "tv.ability_parser",
                "Background parser task started"
            );

            loop {
                notify.notified().await;

                while state.parse_pending.load(Ordering::SeqCst) {
                    tokio::time::sleep(DEBOUNCE_DELAY).await;

                    let should_run = {
                        let last_trigger =
                            state.last_trigger.lock().expect("last_trigger lock poisoned");
                        last_trigger.is_some_and(|t| t.elapsed() >= DEBOUNCE_DELAY)
                    };

                    if should_run {
                        state.parse_pending.store(false, Ordering::SeqCst);

                        let directory = {
                            state
                                .tabula_directory
                                .lock()
                                .expect("tabula_directory lock poisoned")
                                .clone()
                        };

                        if let Some(dir) = directory {
                            state.run_parser(&app_handle, &dir).await;
                        }
                    }
                }
            }
        });
    }

    async fn run_parser(&self, app_handle: &AppHandle, tabula_dir: &Path) {
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

        let result = self.execute_parser(tabula_dir).await;

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

    async fn execute_parser(&self, tabula_dir: &Path) -> Result<(), String> {
        let project_root = find_project_root(tabula_dir).ok_or_else(|| {
            "Could not find project root (looking for rules_engine/Cargo.toml)".to_string()
        })?;

        let manifest_path = project_root.join("rules_engine").join("Cargo.toml");
        let output_path = tabula_dir.join("parsed_abilities.json");

        tracing::debug!(
            component = "tv.ability_parser",
            manifest_path = %manifest_path.display(),
            tabula_dir = %tabula_dir.display(),
            output_path = %output_path.display(),
            "Executing parser command"
        );

        let output = Command::new("cargo")
            .arg("run")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--bin")
            .arg("parser_v2")
            .arg("--")
            .arg("parse-abilities")
            .arg("--directory")
            .arg(tabula_dir)
            .arg("--output")
            .arg(&output_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to spawn parser process: {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!(
                "Parser exited with code {:?}\nstderr: {}\nstdout: {}",
                output.status.code(),
                stderr,
                stdout
            ))
        }
    }
}

impl Default for AbilityParserState {
    fn default() -> Self {
        Self::new()
    }
}

fn find_project_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.to_path_buf();
    loop {
        let cargo_toml = current.join("rules_engine").join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}
