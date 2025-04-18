use std::cell::RefCell;
use std::fmt::Write;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{LazyLock, Mutex};

use action_data::game_action::GameAction;
use backtrace::Backtrace;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::command::CommandSequence;
use display_data::request_data::{
    ConnectRequest, ConnectResponse, PerformActionRequest, PerformActionResponse,
};
use game_creation::new_battle;
use uuid::Uuid;

use crate::{error_message, handle_battle_action};

static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleData>>> = LazyLock::new(|| Mutex::new(None));

thread_local! {
    static PANIC_INFO: RefCell<Option<(String, String, Backtrace)>> = const { RefCell::new(None) };
}

pub fn connect(request: &ConnectRequest) -> ConnectResponse {
    let metadata = request.metadata;
    let commands = catch_panic(
        AssertUnwindSafe(|| {
            let id = BattleId(Uuid::new_v4());
            let battle = new_battle::create_and_start(id);
            let commands = renderer::connect(&battle);
            *CURRENT_BATTLE.lock().unwrap() = Some(battle);
            commands
        }),
        None,
    );
    ConnectResponse { metadata, commands }
}

pub fn perform_action(request: &PerformActionRequest) -> PerformActionResponse {
    let battle_data = CURRENT_BATTLE.lock().unwrap().clone();
    let metadata = request.metadata;
    let commands = catch_panic(
        AssertUnwindSafe(|| {
            let mut battle = match &battle_data {
                Some(battle) => battle.clone(),
                None => panic!("No battle found"),
            };
            battle.animations = Some(AnimationData::default());
            let commands = match request.action {
                GameAction::BattleAction(action) => {
                    handle_battle_action::execute(&mut battle, PlayerName::User, action)
                }
                _ => todo!("Implement other actions"),
            };
            *CURRENT_BATTLE.lock().unwrap() = Some(battle);
            commands
        }),
        battle_data.as_ref(),
    );
    PerformActionResponse { metadata, commands }
}

fn catch_panic<F>(f: F, battle: Option<&BattleData>) -> CommandSequence
where
    F: FnOnce() -> CommandSequence + panic::UnwindSafe,
{
    // Clear any previous panic info
    PANIC_INFO.with(|info| {
        *info.borrow_mut() = None;
    });

    // Set panic hook to capture backtrace
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|panic_info| {
        let location_str = match panic_info.location() {
            Some(location) => format!("{}:{}", location.file(), location.line()),
            None => "unknown location".to_string(),
        };

        let panic_msg = format!("{}", panic_info);
        let backtrace = Backtrace::new();

        PANIC_INFO.with(|info| {
            *info.borrow_mut() = Some((location_str, panic_msg, backtrace));
        });
    }));

    let result = panic::catch_unwind(f);

    // Restore the original panic hook
    panic::set_hook(prev_hook);

    match result {
        Ok(commands) => commands,
        Err(panic_error) => {
            // Extract a more meaningful error message from the panic payload
            let panic_msg = match panic_error.downcast_ref::<&'static str>() {
                Some(s) => s.to_string(),
                None => match panic_error.downcast_ref::<String>() {
                    Some(s) => s.to_string(),
                    None => "Unknown panic".to_string(),
                },
            };

            let mut error_message = PANIC_INFO.with(|info| {
                if let Some((location, info, backtrace)) = &*info.borrow() {
                    let backtrace_str = format!("{:?}", backtrace);
                    let filtered_backtrace = filter_backtrace(&backtrace_str);

                    format!(
                        "Error: {} at {}\n\nError details for developers:\n{}\n{}",
                        panic_msg, location, info, filtered_backtrace
                    )
                } else {
                    format!("Error: {}\n\nNo backtrace available", panic_msg)
                }
            });

            // Limit the length of the error message to avoid overwhelming the UI
            if error_message.len() > 3000 {
                error_message = format!("{}...(truncated)", &error_message[..3000]);
            }

            match battle {
                Some(battle_data) => {
                    error_message::display_error_message(battle_data, error_message)
                }
                None => {
                    // Create a dummy battle if none exists
                    let id = BattleId(Uuid::new_v4());
                    let dummy_battle = new_battle::create_and_start(id);
                    error_message::display_error_message(&dummy_battle, error_message)
                }
            }
        }
    }
}

fn filter_backtrace(backtrace: &str) -> String {
    let mut result = String::new();

    for line in backtrace.lines() {
        if !(line.contains("rustc") || line.contains(".cargo")) {
            writeln!(result, "{}", line).ok();
        }
    }

    result
}
