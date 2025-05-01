use std::cell::RefCell;
use std::fmt::Write;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{LazyLock, Mutex};

use action_data::game_action_data::GameAction;
use backtrace::Backtrace;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use battle_data::battle_player::player_data::PlayerType;
use core_data::identifiers::{BattleId, UserId};
use display::rendering::{battle_rendering, renderer};
use display_data::command::CommandSequence;
use display_data::request_data::{ConnectRequest, ConnectResponse, PerformActionRequest};
use game_creation::new_battle;
use logging::battle_trace;
use tokio::task;
use tracing::{error, info};
use uuid::Uuid;

use crate::{debug_actions, error_message, handle_battle_action};

static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleData>>> = LazyLock::new(|| Mutex::new(None));

thread_local! {
    static PANIC_INFO: RefCell<Option<(String, String, Backtrace)>> = const { RefCell::new(None) };
}

pub fn connect(request: &ConnectRequest) -> ConnectResponse {
    let metadata = request.metadata;
    let commands = catch_panic(
        AssertUnwindSafe(|| {
            connect_internal(request.metadata.user_id);
            Some(renderer::connect(
                CURRENT_BATTLE.lock().unwrap().as_ref().unwrap(),
                request.metadata.user_id,
            ))
        }),
        None,
    );
    ConnectResponse { metadata, commands: commands.unwrap() }
}

pub fn poll(user_id: UserId) -> Option<CommandSequence> {
    if let Some(commands) = handle_battle_action::poll(user_id) {
        battle_trace::write_commands(None, "Returning async command sequence", &commands);
        return Some(commands);
    }
    None
}

pub fn perform_action(request: PerformActionRequest) {
    task::spawn_blocking(move || perform_action_internal(&request));
}

fn connect_internal(user_id: UserId) {
    let mut battle_lock = CURRENT_BATTLE.lock().unwrap();

    if let Some(battle) = battle_lock.as_ref() {
        if let PlayerType::User(current_user_id) = &battle.player_one.player_type {
            if *current_user_id == user_id {
                // Create a new battle if the user id matches the current "user"
                // player
                info!(?user_id, "Restarting current battle");
                *battle_lock =
                    Some(new_battle::create_and_start(user_id, BattleId(Uuid::new_v4())));
                battle_trace::clear_log_file();
                return;
            }
        }

        if let PlayerType::User(enemy_user_id) = &battle.player_two.player_type {
            if *enemy_user_id == user_id {
                return;
            }
        }

        // User is neither the "user" player nor the "enemy" player and a battle
        // already exists; set them as the enemy player
        info!(?user_id, "Joining current battle");
        let mut updated_battle = battle.clone();
        updated_battle.player_two.player_type = PlayerType::User(user_id);
        *battle_lock = Some(updated_battle);
        return;
    }

    // No current battle, create a new one
    info!(?user_id, "No current battle, creating");
    *battle_lock = Some(new_battle::create_and_start(user_id, BattleId(Uuid::new_v4())));
}

fn perform_action_internal(request: &PerformActionRequest) {
    let battle_data = CURRENT_BATTLE.lock().unwrap().clone();
    let metadata = request.metadata;
    let user_id = metadata.user_id;
    let panic_commands = catch_panic(
        AssertUnwindSafe(|| {
            let mut battle = match &battle_data {
                Some(battle) => battle.clone(),
                None => panic!("No battle found"),
            };
            battle.animations = Some(AnimationData::default());
            match request.action {
                GameAction::DebugAction(action) => {
                    let player = renderer::player_name_for_user(&battle, user_id);
                    debug_actions::execute(&mut battle, user_id, player, action);
                    handle_battle_action::append_update(
                        user_id,
                        renderer::connect(&battle, user_id),
                    );
                }
                GameAction::BattleAction(action) => {
                    let player = renderer::player_name_for_user(&battle, user_id);
                    handle_battle_action::execute(&mut battle, user_id, player, action);
                }
                GameAction::OpenPanel(address) => {
                    battle_rendering::open_panel(address);
                    handle_battle_action::append_update(
                        user_id,
                        renderer::connect(&battle, user_id),
                    );
                }
                GameAction::CloseCurrentPanel => {
                    battle_rendering::close_current_panel();
                    handle_battle_action::append_update(
                        user_id,
                        renderer::connect(&battle, user_id),
                    );
                }
            };
            *CURRENT_BATTLE.lock().unwrap() = Some(battle);
            None
        }),
        battle_data.as_ref(),
    );

    if let Some(commands) = panic_commands {
        handle_battle_action::append_update(user_id, commands);
    }
}

fn catch_panic<F>(f: F, battle: Option<&BattleData>) -> Option<CommandSequence>
where
    F: FnOnce() -> Option<CommandSequence> + panic::UnwindSafe,
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

            error!("Captured panic: {}", error_message);

            match battle {
                Some(battle_data) => {
                    Some(error_message::display_error_message(battle_data, error_message))
                }
                None => {
                    // Create a dummy battle if none exists
                    let id = BattleId(Uuid::new_v4());
                    let dummy_battle = new_battle::create_and_start(UserId::default(), id);
                    Some(error_message::display_error_message(&dummy_battle, error_message))
                }
            }
        }
    }
}

fn filter_backtrace(backtrace: &str) -> String {
    let mut result = String::new();
    let skip = [
        "rustc",
        ".cargo",
        "backtrace",
        "catch_panic",
        "rust_panic_with_hook",
        "panic_fmt",
        "rust_begin_unwind",
        "begin_panic_handler",
    ];

    for line in backtrace.lines() {
        if !skip.iter().any(|s| line.contains(s)) {
            writeln!(result, "{}", line).ok();
        }
    }

    result
}
