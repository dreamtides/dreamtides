use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_tracing::BattleTraceEvent;
use battle_data::debug_snapshots::debug_battle_data::DebugBattleData;
use display_data::command::CommandSequence;
use serde::Serialize;
use serde_json;
use tracing::error;

/// Macro for adding a tracing event to a battle.
///
/// This macro does two things:
/// 1. If tracing is enabled for the battle, it records an event in the battle's
///    trace history.
/// 2. It emits a debug-level trace event via the 'tracing' crate.
///
/// Arguments:
/// - `$message`: A message describing the event.
/// - `$battle`: The battle data, must be a mutable reference.
/// - The remaining arguments are symbols to include in the trace and can take
///   either of two forms:
///
///   - A list of variable names
///   - A list of expressions with the form `name = expr`
///
///   (Note that these forms cannot be combined)
///
/// Example:
/// ```rust
/// // With simple variables:
/// battle_trace!("Drawing cards", battle, player, count);
///
/// // With expressions:
/// battle_trace!("Resolving card", battle, card_id = card_id, controller = source.controller());
/// ```
#[macro_export]
macro_rules! battle_trace {
    ($message:expr, $battle:expr) => {{
        tracing::debug!($message);

        if $battle.tracing.is_some() {
            let event = battle_data::battle::battle_tracing::BattleTraceEvent {
                m: $message.to_string(),
                vs: String::new(),
                values: std::collections::BTreeMap::new(),
                snapshot: $battle.debug_snapshot(),
            };
            $battle.add_tracing_event(event.clone());
            $crate::battle_trace::write_battle_event(&event);
        }
    }};
    ($message:expr, $battle:expr, $($key:ident),* $(,)?) => {{
        $( let $key = &$key; )*
        tracing::debug!(message = %$message, $($key = ?$key),*);

        if $battle.tracing.is_some() {
            let mut values = std::collections::BTreeMap::new();
            let mut values_string = String::new();
            $(
                values.insert(stringify!($key).to_string(), format!("{:?}", $key));
                values_string.push_str(&format!("{}: {:?}, ", stringify!($key), $key));
            )*

            let event = battle_data::battle::battle_tracing::BattleTraceEvent {
                m: $message.to_string(),
                vs: values_string,
                values,
                snapshot: $battle.debug_snapshot(),
            };
            $battle.add_tracing_event(event.clone());
            $crate::battle_trace::write_battle_event(&event);
        }
    }};
    ($message:expr, $battle:expr, $($key:ident = $value:expr),* $(,)?) => {{
        tracing::debug!(message = %$message, $($key = ?$value),*);

        if $battle.tracing.is_some() {
            let mut values = std::collections::BTreeMap::new();
            let mut values_string = String::new();
            $(
                values.insert(stringify!($key).to_string(), format!("{:?}", $value));
                values_string.push_str(&format!("{}: {:?}, ", stringify!($key), $value));
            )*

            let event = battle_data::battle::battle_tracing::BattleTraceEvent {
                m: $message.to_string(),
                vs: values_string,
                values,
                snapshot: $battle.debug_snapshot(),
            };
            $battle.add_tracing_event(event.clone());
            $crate::battle_trace::write_battle_event(&event);
        }
    }};
    ($message:expr, $battle:expr, $($simple_key:ident),+ $(,)? $($complex_key:ident = $complex_value:expr),+ $(,)?) => {{
        $( let $simple_key = &$simple_key; )*
        tracing::debug!(message = %$message,
            $($simple_key = ?$simple_key,)* $($complex_key = ?$complex_value),*);

        if $battle.tracing.is_some() {
            let mut values = std::collections::BTreeMap::new();
            let mut values_string = String::new();
            $(
                values.insert(stringify!($simple_key).to_string(), format!("{:?}", $simple_key));
                values_string.push_str(&format!("{}: {:?}, ", stringify!($simple_key), $simple_key));
            )*
            $(
                values.insert(stringify!($complex_key).to_string(), format!("{:?}", $complex_value));
                values_string.push_str(&format!("{}: {:?}, ", stringify!($complex_key), $complex_value));
            )*

            let event = battle_data::battle::battle_tracing::BattleTraceEvent {
                m: $message.to_string(),
                vs: values_string,
                values,
                snapshot: $battle.debug_snapshot(),
            };
            $battle.add_tracing_event(event.clone());
            $crate::battle_trace::write_battle_event(&event);
        }
    }};
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandTraceEvent {
    pub m: String,
    pub snapshot: Option<DebugBattleData>,
    pub sequence: CommandSequence,
}

pub fn write_commands(
    battle: Option<&BattleData>,
    message: &'static str,
    sequence: &CommandSequence,
) {
    let snapshot = battle.filter(|b| b.tracing.is_some()).map(|b| b.debug_snapshot());
    let event = CommandTraceEvent { m: message.to_string(), snapshot, sequence: sequence.clone() };
    match serde_json::to_string_pretty(&event) {
        Ok(json) => write_json_to_log_file(&json),
        Err(e) => error!("Failed to serialize CommandSequence: {}", e),
    }
}

pub fn write_battle_event(event: &BattleTraceEvent) {
    match serde_json::to_string_pretty(event) {
        Ok(json) => write_json_to_log_file(&json),
        Err(e) => error!("Failed to serialize event: {}", e),
    }
}

fn write_json_to_log_file(json_str: &str) {
    let log_path = Path::new("log.json");

    if !log_path.exists() {
        match File::create(log_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(format!("[\n{}\n]", json_str).as_bytes()) {
                    error!("Failed to write to log.json: {}", e);
                }
            }
            Err(e) => error!("Failed to create log.json: {}", e),
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
                            error!("Failed to append to log.json: {}", e);
                        }
                        return;
                    }
                }
                reset_file(&mut file, json_str);
            }
            Err(_) => reset_file(&mut file, json_str),
        },
        Err(e) => error!("Failed to open log.json for appending: {}", e),
    }
}

fn reset_file(file: &mut File, json_str: &str) {
    if file.seek(SeekFrom::Start(0)).is_err() || file.set_len(0).is_err() {
        error!("Failed to reset file");
        return;
    }

    if let Err(e) = file.write_all(format!("[\n{}\n]", json_str).as_bytes()) {
        error!("Failed to write to log.json: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use battle_data::battle::battle_data::BattleData;
    use battle_data::battle::battle_status::BattleStatus;
    use battle_data::battle::battle_tracing::BattleTracing;
    use battle_data::battle::battle_turn_step::BattleTurnStep;
    use battle_data::battle::request_context::RequestContext;
    use battle_data::battle::turn_data::TurnData;
    use battle_data::battle_cards::all_cards::AllCards;
    use battle_data::battle_player::player_data::{PlayerData, PlayerType};
    use core_data::identifiers::{BattleId, UserId};
    use core_data::numerics::{Energy, Points, Spark, TurnId};
    use core_data::types::PlayerName;
    use display_data::command::CommandSequence;
    use rand_xoshiro::rand_core::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;
    use uuid::Uuid;

    fn create_test_battle() -> BattleData {
        BattleData {
            id: BattleId(Uuid::new_v4()),
            request_context: RequestContext::UserRequest,
            player_one: PlayerData {
                name: PlayerName::One,
                player_type: PlayerType::User(UserId::default()),
                points: Points(0),
                current_energy: Energy(2),
                produced_energy: Energy(2),
                spark_bonus: Spark(0),
            },
            player_two: PlayerData {
                name: PlayerName::Two,
                player_type: PlayerType::User(UserId::default()),
                points: Points(0),
                current_energy: Energy(2),
                produced_energy: Energy(2),
                spark_bonus: Spark(0),
            },
            cards: AllCards::default(),
            status: BattleStatus::Playing,
            turn: TurnData { active_player: PlayerName::One, turn_id: TurnId(1) },
            step: BattleTurnStep::Main,
            rng: Xoshiro256PlusPlus::seed_from_u64(12345),
            animations: None,
            prompt: None,
            prompt_resume_action: None,
            tracing: Some(BattleTracing::default()),
        }
    }

    #[test]
    fn test_battle_trace_with_no_values() {
        let mut battle = create_test_battle();
        battle_trace!("Something happened", battle);

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].m, "Something happened");
        assert!(events[0].values.is_empty());
        assert_eq!(events[0].vs, "");
    }

    #[test]
    fn test_battle_trace_with_values() {
        let mut battle = create_test_battle();
        let player = PlayerName::One;
        let count = 2;

        battle_trace!("Drawing cards", battle, player, count);

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].m, "Drawing cards");
        assert_eq!(events[0].values.get("player").unwrap(), "One");
        assert_eq!(events[0].values.get("count").unwrap(), "2");
        assert!(events[0].vs.contains("player: One"));
        assert!(events[0].vs.contains("count: 2"));
    }

    #[test]
    fn test_battle_trace_multiple_events() {
        let mut battle = create_test_battle();

        battle_trace!("First event", battle);

        let damage = 5;
        let target = "enemy character";
        battle_trace!("Damage dealt", battle, damage, target);

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 2);

        assert_eq!(events[0].m, "First event");
        assert!(events[0].values.is_empty());
        assert_eq!(events[0].vs, "");

        assert_eq!(events[1].m, "Damage dealt");
        assert_eq!(events[1].values.get("damage").unwrap(), "5");
        assert_eq!(events[1].values.get("target").unwrap(), "\"enemy character\"");
        assert!(events[1].vs.contains("damage: 5"));
        assert!(events[1].vs.contains("target: \"enemy character\""));
    }

    #[test]
    fn test_values_string_format() {
        let mut battle = create_test_battle();
        let number = 42;
        let text = "sample text";

        battle_trace!("Format test", battle, number, text);

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 1);

        let expected_format = "number: 42, text: \"sample text\", ";
        assert_eq!(events[0].vs, expected_format);
    }

    #[test]
    fn test_battle_trace_with_expressions() {
        let mut battle = create_test_battle();
        let player = PlayerName::One;
        let count = 2;

        // Simple usage
        battle_trace!("Simple trace", battle, player, count);

        // Using expressions
        battle_trace!(
            "With expressions",
            battle,
            player_name = format!("{:?}", player),
            doubled_count = count * 2
        );

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 2);

        assert_eq!(events[0].m, "Simple trace");
        assert_eq!(events[0].values.get("player").unwrap(), "One");
        assert_eq!(events[0].values.get("count").unwrap(), "2");

        assert_eq!(events[1].m, "With expressions");
        assert_eq!(events[1].values.get("player_name").unwrap(), "\"One\"");
        assert_eq!(events[1].values.get("doubled_count").unwrap(), "4");
    }

    #[test]
    fn test_write_commands() {
        let sequence = CommandSequence { groups: vec![] };
        let battle = create_test_battle();

        let log_path = "log.json";
        if fs::metadata(log_path).is_ok() {
            fs::remove_file(log_path).unwrap();
        }

        super::write_commands(Some(&battle), "Command sequence", &sequence);

        assert!(fs::metadata(log_path).is_ok());
        let contents = fs::read_to_string(log_path).unwrap();
        assert!(contents.contains("groups"));

        fs::remove_file(log_path).unwrap();
    }
}
