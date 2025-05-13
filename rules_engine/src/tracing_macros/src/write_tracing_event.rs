use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use battle_queries::debug_snapshot::debug_battle_snapshot;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_trace::battle_tracing::BattleTraceEvent;
use battle_state::debug::debug_battle_state::DebugBattleState;
use display_data::command::CommandSequence;
use serde::Serialize;
use serde_json;
use tracing::error;

pub fn write_battle_event(
    battle: &mut BattleState,
    message: String,
    values: BTreeMap<String, String>,
) {
    let snapshot = debug_battle_snapshot::capture(battle);

    if let Some(tracing) = &mut battle.tracing {
        let values_string = values.iter().fold(String::new(), |mut acc, (k, v)| {
            let _ = write!(acc, "{}: {}, ", k, v);
            acc
        });
        let event = BattleTraceEvent { m: message, vs: values_string, values, snapshot };

        write_event_to_log_file(&event);

        if tracing.turn != battle.turn.turn_id {
            tracing.turn = battle.turn.turn_id;
            tracing.current.clear();
        }

        tracing.current.push(event);
    }
}

pub fn write_panic_snapshot(
    battle: &BattleState,
    message: String,
    values: BTreeMap<String, String>,
) {
    let snapshot = debug_battle_snapshot::capture(battle);
    let values_string = values.iter().fold(String::new(), |mut acc, (k, v)| {
        let _ = write!(acc, "{}: {}, ", k, v);
        acc
    });
    let event =
        BattleTraceEvent { m: format!("PANIC: {}", message), vs: values_string, values, snapshot };
    write_event_to_log_file(&event);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandTraceEvent {
    pub m: String,
    pub snapshot: Option<DebugBattleState>,
    pub sequence: CommandSequence,
}

pub fn write_commands(
    battle: Option<&BattleState>,
    message: &'static str,
    sequence: &CommandSequence,
) {
    let snapshot = battle.filter(|b| b.tracing.is_some()).map(debug_battle_snapshot::capture);
    let event = CommandTraceEvent { m: message.to_string(), snapshot, sequence: sequence.clone() };
    match serde_json::to_string_pretty(&event) {
        Ok(json) => write_json_to_log_file(&json),
        Err(e) => error!("Failed to serialize CommandSequence: {}", e),
    }
}

pub fn clear_log_file() {
    let log_path = Path::new("dreamtides.json");
    if log_path.exists() {
        fs::remove_file(log_path).expect("Failed to clear dreamtides.json");
    }
}

fn write_event_to_log_file(event: &BattleTraceEvent) {
    match serde_json::to_string_pretty(event) {
        Ok(json) => write_json_to_log_file(&json),
        Err(e) => error!("Failed to serialize event: {}", e),
    }
}

fn write_json_to_log_file(json_str: &str) {
    let log_path = Path::new("dreamtides.json");

    if !log_path.exists() {
        match File::create(log_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(format!("[\n{}\n]", json_str).as_bytes()) {
                    error!("Failed to write to dreamtides.json: {}", e);
                }
            }
            Err(e) => error!("Failed to create dreamtides.json: {}", e),
        }
        return;
    }

    match OpenOptions::new().read(true).write(true).open(log_path) {
        Ok(mut file) => match file.metadata() {
            Ok(metadata) => {
                if metadata.len() > 0 {
                    if file.seek(SeekFrom::End(-1)).is_err() {
                        reset_file(&mut file, json_str);
                        return;
                    }

                    let mut last_char = [0u8; 1];
                    if file.read_exact(&mut last_char).is_err() {
                        reset_file(&mut file, json_str);
                        return;
                    }

                    if last_char[0] == b']' {
                        if file.seek(SeekFrom::End(-1)).is_err() {
                            reset_file(&mut file, json_str);
                            return;
                        }

                        if let Err(e) = file.write_all(format!(",\n{}\n]", json_str).as_bytes()) {
                            error!("Failed to append to dreamtides.json: {}", e);
                        }
                        return;
                    }
                }
                reset_file(&mut file, json_str);
            }
            Err(_) => reset_file(&mut file, json_str),
        },
        Err(e) => error!("Failed to open dreamtides.json for appending: {}", e),
    }
}

fn reset_file(file: &mut File, json_str: &str) {
    if file.seek(SeekFrom::Start(0)).is_err() || file.set_len(0).is_err() {
        error!("Failed to reset file");
        return;
    }

    if let Err(e) = file.write_all(format!("[\n{}\n]", json_str).as_bytes()) {
        error!("Failed to write to dreamtides.json: {}", e);
    }
}
