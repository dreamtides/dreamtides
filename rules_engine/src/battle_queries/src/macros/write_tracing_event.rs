use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::SystemTime;

use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle_trace::battle_tracing::BattleTraceEvent;
use battle_state::debug::debug_battle_state::DebugBattleState;
use chrono::{DateTime, Local};
use display_data::command::CommandSequence;
use serde::Serialize;
use serde_json;
use strum::IntoDiscriminant;
use tracing::{debug, error};

use crate::debug_snapshot::debug_battle_snapshot;

pub fn write_battle_event(
    battle: &mut BattleState,
    message: String,
    values: BTreeMap<String, String>,
) {
    let snapshot = debug_battle_snapshot::capture(battle);

    if let Some(tracing) = &mut battle.tracing {
        let values_string = values.iter().fold(String::new(), |mut acc, (k, v)| {
            let _ = write!(acc, "{k}: {v}, ");
            acc
        });
        let timestamp = format_current_time();
        let event = BattleTraceEvent { m: message, vs: values_string, values, snapshot, timestamp };

        write_event_to_log_file(&event, &battle.request_context);

        if tracing.turn != battle.turn.turn_id {
            tracing.turn = battle.turn.turn_id;
            tracing.current.clear();
        }

        tracing.current.push(event);
    }
}

#[expect(clippy::print_stderr)]
pub fn write_panic_snapshot(
    battle: &BattleState,
    message: String,
    values: BTreeMap<String, String>,
) {
    let snapshot = debug_battle_snapshot::capture(battle);
    let values_string = values.iter().fold(String::new(), |mut acc, (k, v)| {
        let _ = write!(acc, "{k}: {v}, ");
        acc
    });
    let timestamp = format_current_time();
    let event = BattleTraceEvent {
        m: format!("PANIC: {message}"),
        vs: values_string,
        values,
        snapshot,
        timestamp,
    };

    if !write_event_to_log_file(&event, &battle.request_context) {
        eprintln!("Failed to write panic snapshot to log file");
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeserializationPanicEvent {
    pub m: String,
    pub snapshot: DebugBattleState,
    pub panic_action_index: usize,
    pub total_actions: usize,
    pub panic_info: String,
    pub action_history: Vec<serde_json::Value>,
    pub panic_details: serde_json::Map<String, serde_json::Value>,
    pub timestamp: String,
}

pub fn write_deserialization_panic(
    battle: &BattleState,
    panic_action_index: usize,
    total_actions: usize,
    panic_info: String,
    action_history: Vec<serde_json::Value>,
    panic_details: serde_json::Map<String, serde_json::Value>,
) {
    let snapshot = debug_battle_snapshot::capture(battle);
    let timestamp = format_current_time();
    let event = DeserializationPanicEvent {
        m: format!(
            "PANIC: Deserialization panic at action {panic_action_index} of {total_actions}"
        ),
        snapshot,
        panic_action_index,
        total_actions,
        panic_info,
        action_history,
        panic_details,
        timestamp,
    };

    match serde_json::to_string_pretty(&event) {
        Ok(json) => {
            write_json_to_log_file(&json, &battle.request_context);
        }
        Err(e) => {
            error!("Failed to serialize DeserializationPanicEvent: {}", e);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct CommandTraceEvent {
    pub m: String,
    pub snapshot: Option<DebugBattleState>,
    pub sequence: CommandSequence,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
struct AnimationTraceEvent {
    pub m: String,
    pub snapshot: DebugBattleState,
    pub step_names: Vec<String>,
    pub timestamp: String,
}

pub fn write_animations(battle: &BattleState, animations: &AnimationData) {
    let animation_names: Vec<String> = animations
        .steps
        .iter()
        .map(|step| format!("{:?}", step.animation.discriminant()))
        .collect();
    let names = format!("[{}]", animation_names.join(", "));
    debug!(?names, "Playing animations");

    let snapshot = debug_battle_snapshot::capture(battle);
    let timestamp = format_current_time();
    let event = AnimationTraceEvent {
        m: format!("Playing animations: {names}"),
        snapshot,
        step_names: animation_names,
        timestamp,
    };
    match serde_json::to_string_pretty(&event) {
        Ok(json) => {
            write_json_to_log_file(&json, &battle.request_context);
        }
        Err(e) => {
            error!("Failed to serialize CommandSequence: {}", e);
        }
    }
}

pub fn write_commands(sequence: &CommandSequence, request_context: &RequestContext) {
    let command_names: Vec<String> = sequence
        .groups
        .iter()
        .flat_map(|group| &group.commands)
        .map(|command| format!("{:?}", command.discriminant()))
        .collect();
    let names = format!("[{}]", command_names.join(", "));
    debug!(?names, "Writing commands");

    let timestamp = format_current_time();
    let event = CommandTraceEvent {
        m: format!("Writing commands: {names}"),
        snapshot: None,
        sequence: sequence.clone(),
        timestamp,
    };
    match serde_json::to_string_pretty(&event) {
        Ok(json) => {
            write_json_to_log_file(&json, request_context);
        }
        Err(e) => {
            error!("Failed to serialize CommandSequence: {}", e);
        }
    }
}

fn format_current_time() -> String {
    let now = SystemTime::now();
    let datetime: DateTime<Local> = now.into();
    datetime.format("%Y-%m-%d %H:%M:%S%.3f %z").to_string()
}

pub fn clear_log_file(request_context: &RequestContext) {
    let Some(log_path) = get_log_file_path(request_context) else {
        return;
    };

    if log_path.exists() {
        if let Err(e) = fs::remove_file(&log_path) {
            error!(?log_path, "Failed to remove dreamtides.json: {}", e);
        }
    }
}

/// Writes an event to the log file. Returns true if the event was written.
fn write_event_to_log_file(event: &BattleTraceEvent, request_context: &RequestContext) -> bool {
    match serde_json::to_string_pretty(event) {
        Ok(json) => write_json_to_log_file(&json, request_context),
        Err(e) => {
            error!("Failed to serialize event: {}", e);
            false
        }
    }
}

/// Writes a JSON string to the log file. Returns true if the JSON was written.
fn write_json_to_log_file(json_str: &str, request_context: &RequestContext) -> bool {
    let Some(log_path) = get_log_file_path(request_context) else {
        return false;
    };

    if !log_path.exists() {
        match File::create(&log_path) {
            Ok(mut file) => match file.write_all(format!("[\n{json_str}\n]").as_bytes()) {
                Ok(_) => debug!(?log_path, "Created dreamtides.json"),
                Err(e) => {
                    error!(?log_path, "Failed to write to dreamtides.json: {}", e);
                    return false;
                }
            },
            Err(e) => {
                error!(?log_path, "Failed to create dreamtides.json: {}", e);
                return false;
            }
        }
    }

    match OpenOptions::new().read(true).write(true).open(&log_path) {
        Ok(mut file) => match file.metadata() {
            Ok(metadata) => {
                if metadata.len() > 0 {
                    if file.seek(SeekFrom::End(-1)).is_err() {
                        return reset_file(&mut file, json_str);
                    }

                    let mut last_char = [0u8; 1];
                    if file.read_exact(&mut last_char).is_err() {
                        return reset_file(&mut file, json_str);
                    }

                    if last_char[0] == b']' {
                        if file.seek(SeekFrom::End(-1)).is_err() {
                            return reset_file(&mut file, json_str);
                        }

                        if let Err(e) = file.write_all(format!(",\n{json_str}\n]").as_bytes()) {
                            error!(?log_path, "Failed to append to dreamtides.json: {}", e);
                        }
                        return false;
                    }
                }
                reset_file(&mut file, json_str)
            }
            Err(_) => reset_file(&mut file, json_str),
        },
        Err(e) => {
            error!(?log_path, "Failed to open dreamtides.json for appending: {}", e);
            false
        }
    }
}

fn get_log_file_path(request_context: &RequestContext) -> Option<PathBuf> {
    let log_directory = request_context.logging_options.log_directory.as_ref()?;
    Some(log_directory.join("dreamtides.json"))
}

/// Resets the file to the beginning and writes the JSON string to it. Returns
/// true if the file was written successfully.
fn reset_file(file: &mut File, json_str: &str) -> bool {
    if file.seek(SeekFrom::Start(0)).is_err() || file.set_len(0).is_err() {
        error!("Failed to reset file");
        return false;
    }

    if let Err(e) = file.write_all(format!("[\n{json_str}\n]").as_bytes()) {
        error!("Failed to write to dreamtides.json: {}", e);
        return false;
    }

    true
}
