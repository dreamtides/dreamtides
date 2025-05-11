use std::cell::RefCell;
use std::fmt::Write;
use std::panic::{self};
use std::path::PathBuf;

use action_data::game_action_data::GameAction;
use backtrace::Backtrace;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_animations::animation_data::AnimationData;
use battle_data_old::battle_player::player_data::PlayerType;
use core_data::identifiers::{BattleId, QuestId, UserId};
use database::save_file::SaveFile;
use database::sqlite_database::{self, Database};
use display::rendering::{battle_rendering, renderer};
use display_data::command::CommandSequence;
use display_data::request_data::{ConnectRequest, ConnectResponse, PerformActionRequest};
use game_creation_old::new_battle;
use logging::battle_trace;
use tokio::task;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    debug_actions, deserialize_save_file, error_message, handle_battle_action, serialize_save_file,
};

thread_local! {
    static PANIC_INFO: RefCell<Option<(String, String, Backtrace)>> = const { RefCell::new(None) };
}

pub fn connect(request: &ConnectRequest) -> ConnectResponse {
    let metadata = request.metadata;
    let result = catch_panic(|| connect_internal(request));
    let commands = match result {
        Ok(commands) => commands,
        Err(error) => error_message::display_error_message(None, error),
    };
    ConnectResponse { metadata, commands }
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

fn connect_internal(request: &ConnectRequest) -> CommandSequence {
    let user_id = request.metadata.user_id;
    let persistent_data_path = &request.persistent_data_path;
    battle_trace::clear_log_file();

    let database = match initialize_database(persistent_data_path) {
        Ok(db) => db,
        Err(error) => return error_message::display_error_message(None, error),
    };

    // Check if this is a multiplayer connection request
    if let Some(vs_opponent) = request.vs_opponent {
        return connect_for_multiplayer(&database, user_id, vs_opponent);
    }

    info!(?user_id, "Loading battle from database");
    match load_battle_from_database(&database, user_id) {
        Ok(LoadBattleResult::ExistingBattle(battle, quest_id)) => {
            if is_user_in_battle(&battle, user_id) {
                renderer::connect(&battle, user_id)
            } else {
                handle_user_not_in_battle(user_id, battle, quest_id, &database, None)
            }
        }
        Ok(LoadBattleResult::NewBattle(battle)) => renderer::connect(&battle, user_id),
        Err(error) => error_message::display_error_message(None, error),
    }
}

/// Handles a connection request for multiplayer games.
///
/// Instead of loading the requesting user's save file, this loads the
/// opponent's save file and joins the battle if possible.
fn connect_for_multiplayer(
    database: &Database,
    user_id: UserId,
    vs_opponent: UserId,
) -> CommandSequence {
    info!(?user_id, ?vs_opponent, "Loading multiplayer battle from opponent's database");

    // Load opponent's save file
    match database.fetch_save(vs_opponent) {
        Ok(Some(save_file)) => {
            match deserialize_save_file::battle(&save_file) {
                Some((battle, quest_id)) => {
                    // Check if the connecting user is already in the battle
                    if is_user_in_battle(&battle, user_id) {
                        return renderer::connect(&battle, user_id);
                    }

                    // If not in the battle, try to join by replacing an AI player
                    handle_user_not_in_battle(
                        user_id,
                        battle,
                        quest_id,
                        database,
                        Some(vs_opponent),
                    )
                }
                None => error_message::display_error_message(
                    None,
                    "No battle found in opponent's save file".to_string(),
                ),
            }
        }
        Ok(None) => error_message::display_error_message(
            None,
            format!("No save file found for opponent ID: {:?}", vs_opponent),
        ),
        Err(error) => error_message::display_error_message(None, error.to_string()),
    }
}

enum LoadBattleResult {
    ExistingBattle(BattleData, QuestId),
    NewBattle(BattleData),
}

fn initialize_database(persistent_data_path: &str) -> Result<Database, String> {
    sqlite_database::initialize(PathBuf::from(persistent_data_path)).map_err(|e| e.to_string())
}

fn load_battle_from_database(
    database: &Database,
    user_id: UserId,
) -> Result<LoadBattleResult, String> {
    match database.fetch_save(user_id) {
        Ok(Some(save_file)) => match crate::deserialize_save_file::battle(&save_file) {
            Some((battle, quest_id)) => Ok(LoadBattleResult::ExistingBattle(battle, quest_id)),
            None => Err("No battle in save file".to_string()),
        },
        Ok(None) => {
            // No save file exists, create a new battle
            info!(?user_id, "No save file found, creating new battle");
            let new_battle = new_battle::create_and_start(user_id, BattleId(Uuid::new_v4()));

            // Save new battle to database
            let quest_id = QuestId(Uuid::new_v4());
            let save_file = serialize_save_file::battle(user_id, quest_id, &new_battle);
            match database.write_save(save_file) {
                Ok(_) => Ok(LoadBattleResult::NewBattle(new_battle)),
                Err(error) => Err(error.to_string()),
            }
        }
        Err(error) => Err(error.to_string()),
    }
}

fn is_user_in_battle(battle: &BattleData, user_id: UserId) -> bool {
    match &battle.player_one.player_type {
        PlayerType::User(id) if *id == user_id => true,
        _ => matches!(&battle.player_two.player_type, PlayerType::User(id) if *id == user_id),
    }
}

fn handle_user_not_in_battle(
    user_id: UserId,
    mut battle: BattleData,
    quest_id: QuestId,
    database: &Database,
    vs_opponent: Option<UserId>,
) -> CommandSequence {
    battle_trace!("User is not a player in this battle", battle, user_id);

    // Check if both players are already human users (but not this user)
    let both_human_players = match (&battle.player_one.player_type, &battle.player_two.player_type)
    {
        (PlayerType::User(id1), PlayerType::User(id2)) => *id1 != user_id && *id2 != user_id,
        _ => false,
    };

    if both_human_players {
        return error_message::display_error_message(
            Some(&battle),
            "Cannot join battle: both players are already human users".to_string(),
        );
    }

    // Replace the first non-human player with this user
    if !matches!(battle.player_one.player_type, PlayerType::User(_)) {
        info!(?user_id, "Replacing player one with user");
        battle.player_one.player_type = PlayerType::User(user_id);
    } else if !matches!(battle.player_two.player_type, PlayerType::User(_)) {
        info!(?user_id, "Replacing player two with user");
        battle.player_two.player_type = PlayerType::User(user_id);
    }

    let save_user_id = vs_opponent.unwrap_or(user_id);
    match save_battle_to_database(database, save_user_id, quest_id, &battle) {
        Ok(_) => renderer::connect(&battle, user_id),
        Err(error) => error_message::display_error_message(None, error),
    }
}

fn save_battle_to_database(
    database: &Database,
    user_id: UserId,
    quest_id: QuestId,
    battle: &BattleData,
) -> Result<(), String> {
    let save_file = serialize_save_file::battle(user_id, quest_id, battle);
    database.write_save(save_file).map_err(|e| e.to_string())
}

fn perform_action_internal(request: &PerformActionRequest) {
    let metadata = request.metadata;
    let user_id = metadata.user_id;
    let result = catch_panic(|| {
        let Ok(database) = sqlite_database::get() else {
            show_error_message(user_id, None, "No database found".to_string());
            return;
        };

        // Use vs_opponent's save file if specified, otherwise use the user's save file
        let save_user_id = request.vs_opponent.unwrap_or(user_id);
        let Ok(Some(save)) = database.fetch_save(save_user_id) else {
            let error_msg = if request.vs_opponent.is_some() {
                format!("No save file found for opponent ID: {:?}", save_user_id)
            } else {
                "No save file found".to_string()
            };
            show_error_message(user_id, None, error_msg);
            return;
        };

        let Some((mut battle, quest_id)) = deserialize_save_file::battle(&save) else {
            show_error_message(user_id, None, "No battle found".to_string());
            return;
        };

        battle.animations = Some(AnimationData::default());
        handle_request_action(request, user_id, save, &mut battle);

        // Always save to the save_user_id (either opponent or user)
        if let Err(error) =
            database.write_save(serialize_save_file::battle(save_user_id, quest_id, &battle))
        {
            show_error_message(user_id, Some(&battle), format!("Failed to save battle: {}", error));
        }
    });

    if let Err(error) = result {
        show_error_message(user_id, None, error);
    }
}

fn handle_request_action(
    request: &PerformActionRequest,
    user_id: UserId,
    save: SaveFile,
    battle: &mut BattleData,
) {
    match request.action {
        GameAction::DebugAction(action) => {
            let player = renderer::player_name_for_user(&*battle, user_id);
            debug_actions::execute(battle, user_id, player, action);
            handle_battle_action::append_update(user_id, renderer::connect(&*battle, user_id));
        }
        GameAction::BattleAction(action) => {
            let player = renderer::player_name_for_user(&*battle, user_id);
            handle_battle_action::execute(battle, user_id, player, action);
        }
        GameAction::Undo(player) => {
            let Some((undone_battle, _)) = deserialize_save_file::undo(&save, player) else {
                show_error_message(
                    user_id,
                    None,
                    "Failed to undo: Battle state not found.".to_string(),
                );
                return;
            };

            *battle = undone_battle;
            handle_battle_action::append_update(user_id, renderer::connect(&*battle, user_id));
        }
        GameAction::OpenPanel(address) => {
            battle_rendering::open_panel(address);
            handle_battle_action::append_update(user_id, renderer::connect(&*battle, user_id));
        }
        GameAction::CloseCurrentPanel => {
            battle_rendering::close_current_panel();
            handle_battle_action::append_update(user_id, renderer::connect(&*battle, user_id));
        }
    };
}

pub fn show_error_message(user_id: UserId, battle: Option<&BattleData>, error_message: String) {
    handle_battle_action::append_update(
        user_id,
        error_message::display_error_message(battle, error_message),
    )
}

fn catch_panic<F, T>(function: F) -> Result<T, String>
where
    F: FnOnce() -> T + panic::UnwindSafe,
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

    let result = panic::catch_unwind(function);

    // Restore the original panic hook
    panic::set_hook(prev_hook);

    match result {
        Ok(value) => Ok(value),
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
            Err(error_message)
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
