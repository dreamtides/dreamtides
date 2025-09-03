use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::panic;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use action_data::game_action_data::GameAction;
use ai_data::game_ai::GameAI;
use backtrace::Backtrace;
use battle_queries::battle_trace;
use battle_queries::macros::write_tracing_event;
use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle_cards::dreamwell_data::Dreamwell;
use battle_state::battle_player::battle_player_state::{
    CreateBattlePlayer, PlayerType, TestDeckName,
};
use core_data::identifiers::{BattleId, QuestId, UserId};
use core_data::initialization_error::InitializationError;
use core_data::types::PlayerName;
use display::display_actions::apply_battle_display_action;
use display::rendering::renderer;
use display_data::command::CommandSequence;
use display_data::request_data::{
    ConnectRequest, ConnectResponse, DebugConfiguration, Metadata, PerformActionRequest,
    PollResponse, PollResponseType,
};
use game_creation::new_battle;
use rand::RngCore;
use state_provider::state_provider::{DefaultStateProvider, PollResult, StateProvider};
use state_provider::test_state_provider::TestStateProvider;
use tabula_ids::card_lists::DreamwellCardIdList;
use tokio::task;
use tracing::{Level, debug, error, info, warn};
use ui_components::display_properties;
use uuid::Uuid;

use crate::{
    debug_actions, deserialize_save_file, error_message, handle_battle_action, serialize_save_file,
};

thread_local! {
    static PANIC_INFO: RefCell<Option<(String, String, Backtrace)>> = const { RefCell::new(None) };
}

static TEST_STATE_PROVIDERS: LazyLock<Mutex<HashMap<Uuid, TestStateProvider>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static DEFAULT_AI_OPPONENT: LazyLock<PlayerType> =
    LazyLock::new(|| PlayerType::Agent(GameAI::MonteCarlo(50)));

#[derive(Debug, Clone)]
pub struct PerformActionBlockingResult {
    pub user_poll_results: Vec<PollResult>,
    pub enemy_poll_results: Vec<PollResult>,
}

/// Connects to the rules engine.
pub fn connect(request: &ConnectRequest, request_context: RequestContext) -> ConnectResponse {
    if let Some(integration_test_id) = request.metadata.integration_test_id {
        debug!(?integration_test_id, "Connecting to integration test");
        let provider = get_test_state_provider(integration_test_id);
        connect_with_provider(provider, request, request_context)
    } else {
        connect_with_provider(DefaultStateProvider, request, request_context)
    }
}

/// Connects to the rules engine with the specified [StateProvider] and returns
/// commands to execute.
pub fn connect_with_provider(
    provider: impl StateProvider + 'static,
    request: &ConnectRequest,
    request_context: RequestContext,
) -> ConnectResponse {
    let metadata = request.metadata;
    let user_id = metadata.user_id;

    provider.store_request_context(user_id, request_context.clone());

    let result = catch_panic_conditionally(&provider, || {
        connect_internal(&provider, request, request_context)
    });
    let commands = match result {
        Ok(commands) => commands,
        Err(error) => error_message::display_error_message(None, &provider, error),
    };

    let response_version = Uuid::new_v4();
    provider.store_last_response_version(user_id, response_version);

    ConnectResponse { metadata, commands, response_version }
}

/// Polls for the result of a game action.
pub fn poll(user_id: UserId, metadata: Metadata) -> Option<PollResponse> {
    if let Some(integration_test_id) = metadata.integration_test_id {
        let provider = get_test_state_provider(integration_test_id);
        poll_with_provider(provider, user_id, metadata)
    } else {
        poll_with_provider(DefaultStateProvider, user_id, metadata)
    }
}

/// Polls for the result of a game action with the specified [StateProvider] .
pub fn poll_with_provider(
    provider: impl StateProvider + 'static,
    user_id: UserId,
    metadata: Metadata,
) -> Option<PollResponse> {
    if let Some(poll_result) = handle_battle_action::poll(&provider, user_id) {
        let request_id = poll_result.request_id;
        let elapsed_msg = provider.get_elapsed_time_message(request_id);

        if request_id.is_some() && elapsed_msg.contains("unknown request ID") {
            error!(?request_id, "Unrecognized request ID in poll response");
        }

        let response_version = if matches!(poll_result.response_type, PollResponseType::Final) {
            let version = Uuid::new_v4();
            provider.store_last_response_version(user_id, version);
            Some(version)
        } else {
            None
        };

        debug!(?elapsed_msg, ?request_id, ?response_version, "Returning poll response");
        return Some(PollResponse {
            metadata: Metadata { user_id, request_id, ..metadata },
            commands: Some(poll_result.commands),
            response_type: poll_result.response_type,
            response_version,
        });
    }
    None
}

/// Performs a game action on another thread.
///
/// Returns immediately after spawning the processing thread. Results will be
/// available via [poll] after the action is complete.
pub fn perform_action(request: PerformActionRequest) {
    let request_id = request.metadata.request_id;
    let user_id = request.metadata.user_id;

    if let Some(integration_test_id) = request.metadata.integration_test_id {
        debug!(?integration_test_id, "Performing action for integration test");
        let provider = get_test_state_provider(integration_test_id);
        provider.store_request_timestamp(request_id, Instant::now());
        task::spawn_blocking(move || {
            if perform_action_internal(&provider, &request) {
                provider.finish_processing(user_id);
            }
        });
    } else {
        let provider = DefaultStateProvider;
        provider.store_request_timestamp(request_id, Instant::now());
        task::spawn_blocking(move || {
            if perform_action_internal(&provider, &request) {
                provider.finish_processing(user_id);
            }
        });
    }
}

/// Performs a game action synchronously and blocks until completion.
///
/// This is designed for test scenarios where the overhead of async execution
/// is not desired. It executes the action immediately and waits for the final
/// result before returning.
pub fn perform_action_blocking(
    provider: impl StateProvider + 'static,
    request: PerformActionRequest,
    enemy_id: Option<UserId>,
) -> PerformActionBlockingResult {
    let user_id = request.metadata.user_id;

    let processing_started = perform_action_internal(&provider, &request);
    if processing_started {
        provider.finish_processing(user_id);
    }

    let mut user_results = Vec::new();
    let mut enemy_results = Vec::new();

    while let Some(poll_result) = handle_battle_action::poll(&provider, user_id) {
        user_results.push(poll_result.clone());
        if matches!(poll_result.response_type, PollResponseType::Final) {
            break;
        }
    }

    if let Some(enemy_user_id) = enemy_id {
        while let Some(poll_result) = handle_battle_action::poll(&provider, enemy_user_id) {
            enemy_results.push(poll_result.clone());
            if matches!(poll_result.response_type, PollResponseType::Final) {
                break;
            }
        }
    }

    PerformActionBlockingResult {
        user_poll_results: user_results,
        enemy_poll_results: enemy_results,
    }
}

fn connect_internal<P: StateProvider + 'static>(
    provider: &P,
    request: &ConnectRequest,
    request_context: RequestContext,
) -> CommandSequence {
    let user_id = request.metadata.user_id;
    let persistent_data_path = &request.persistent_data_path;
    let streaming_assets_path = &request.streaming_assets_path;
    write_tracing_event::clear_log_file();

    if let Some(ref display_props) = request.display_properties {
        display_properties::store_display_properties(user_id, display_props.clone());
    }

    debug!(">>> Initializing provider with persistent data path: {:?}", persistent_data_path);
    if let Err(errors) = provider.initialize(persistent_data_path, streaming_assets_path) {
        return error_message::display_error_message(
            None,
            provider,
            format_initialization_errors(&errors),
        );
    }

    // Check if this is a multiplayer connection request
    if let Some(vs_opponent) = request.vs_opponent {
        return connect_for_multiplayer(provider, user_id, vs_opponent, request_context);
    } else {
        info!(?user_id, "Loading battle from database");
    }

    match load_battle_from_provider(
        provider,
        user_id,
        request_context,
        request.debug_configuration.as_ref(),
    ) {
        Ok(LoadBattleResult::ExistingBattle(battle, quest_id)) => {
            if is_user_in_battle(&battle, user_id) {
                renderer::connect(&battle, user_id, (*provider).clone(), false)
            } else {
                handle_user_not_in_battle(provider, user_id, battle, quest_id, None)
            }
        }
        Ok(LoadBattleResult::NewBattle(battle)) => {
            renderer::connect(&battle, user_id, (*provider).clone(), false)
        }
        Err(error) => error_message::display_error_message(None, provider, error),
    }
}

/// Handles a connection request for multiplayer games.
///
/// Instead of loading the requesting user's save file, this loads the
/// opponent's save file and joins the battle if possible.
fn connect_for_multiplayer<P: StateProvider + 'static>(
    provider: &P,
    user_id: UserId,
    vs_opponent: UserId,
    request_context: RequestContext,
) -> CommandSequence {
    info!(?user_id, ?vs_opponent, "Loading multiplayer battle from opponent's database");

    match provider.read_save_file(vs_opponent) {
        Ok(Some(save_file)) => {
            match deserialize_save_file::battle(provider, &save_file, request_context) {
                Some((battle, quest_id)) => {
                    // Check if the connecting user is already in the battle
                    if is_user_in_battle(&battle, user_id) {
                        return renderer::connect(&battle, user_id, (*provider).clone(), false);
                    }

                    // If not in the battle, try to join by replacing an AI player
                    handle_user_not_in_battle(
                        provider,
                        user_id,
                        battle,
                        quest_id,
                        Some(vs_opponent),
                    )
                }
                None => error_message::display_error_message(
                    None,
                    provider,
                    "No battle found in opponent's save file".to_string(),
                ),
            }
        }
        Ok(None) => error_message::display_error_message(
            None,
            provider,
            format!("No save file found for opponent ID: {vs_opponent:?}"),
        ),
        Err(errors) => error_message::display_error_message(
            None,
            provider,
            format_initialization_errors(&errors),
        ),
    }
}

enum LoadBattleResult {
    ExistingBattle(BattleState, QuestId),
    NewBattle(BattleState),
}

fn load_battle_from_provider<P: StateProvider + 'static>(
    provider: &P,
    user_id: UserId,
    request_context: RequestContext,
    debug_configuration: Option<&DebugConfiguration>,
) -> Result<LoadBattleResult, String> {
    match provider.read_save_file(user_id) {
        Ok(Some(save_file)) => {
            match deserialize_save_file::battle(provider, &save_file, request_context) {
                Some((battle, quest_id)) => {
                    // provider.clear_undo_stack(battle.id);
                    Ok(LoadBattleResult::ExistingBattle(battle, quest_id))
                }
                None => Err("No battle in save file".to_string()),
            }
        }
        Ok(None) => {
            // No save file exists, create a new battle
            info!(?user_id, "No save file found, creating new battle");
            let battle_id = BattleId(Uuid::new_v4());

            let configuration = debug_configuration.cloned().unwrap_or_default();
            let seed = configuration.seed.unwrap_or_else(|| rand::rng().next_u64());
            let enemy =
                configuration.enemy.as_ref().cloned().unwrap_or(DEFAULT_AI_OPPONENT.clone());

            let deck_name = configuration.deck_override.unwrap_or(TestDeckName::Core11);
            let new_battle = new_battle::create_and_start(
                battle_id,
                provider.tabula(),
                seed,
                Dreamwell::from_card_list(
                    &provider.tabula(),
                    configuration
                        .dreamwell_override
                        .unwrap_or(DreamwellCardIdList::DreamwellBasic5),
                ),
                CreateBattlePlayer { player_type: PlayerType::User(user_id), deck_name },
                CreateBattlePlayer { player_type: enemy, deck_name },
                request_context,
            );

            provider.clear_undo_stack(new_battle.id);
            // Save new battle to database
            let quest_id = QuestId(Uuid::new_v4());
            let save_file = serialize_save_file::battle(user_id, quest_id, &new_battle);
            match provider.write_save_file(save_file) {
                Ok(_) => Ok(LoadBattleResult::NewBattle(new_battle)),
                Err(errors) => Err(format_initialization_errors(&errors)),
            }
        }
        Err(errors) => Err(format_initialization_errors(&errors)),
    }
}

fn is_user_in_battle(battle: &BattleState, user_id: UserId) -> bool {
    match &battle.players.one.player_type {
        PlayerType::User(id) if *id == user_id => true,
        _ => matches!(&battle.players.two.player_type, PlayerType::User(id) if *id == user_id),
    }
}

fn handle_user_not_in_battle<P: StateProvider + 'static>(
    provider: &P,
    user_id: UserId,
    mut battle: BattleState,
    quest_id: QuestId,
    vs_opponent: Option<UserId>,
) -> CommandSequence {
    battle_trace!("User is not a player in this battle, attempting to join", &mut battle, user_id);

    let both_human_players =
        match (&battle.players.one.player_type, &battle.players.two.player_type) {
            (PlayerType::User(id1), PlayerType::User(id2)) => *id1 != user_id && *id2 != user_id,
            _ => false,
        };

    if both_human_players {
        warn!(?user_id, "Both players are human users, replacing player two with user");
        battle.players.two.player_type = PlayerType::User(user_id);
    } else if !matches!(battle.players.one.player_type, PlayerType::User(_)) {
        // Replace the first non-human player with this user
        info!(?user_id, "Replacing player one with user");
        battle.players.one.player_type = PlayerType::User(user_id);
    } else if !matches!(battle.players.two.player_type, PlayerType::User(_)) {
        info!(?user_id, "Replacing player two with user");
        battle.players.two.player_type = PlayerType::User(user_id);
    }

    let save_user_id = vs_opponent.unwrap_or(user_id);
    match save_battle_to_provider(provider, save_user_id, quest_id, &battle) {
        Ok(_) => renderer::connect(&battle, user_id, (*provider).clone(), false),
        Err(error) => error_message::display_error_message(None, provider, error),
    }
}

fn save_battle_to_provider(
    provider: &impl StateProvider,
    user_id: UserId,
    quest_id: QuestId,
    battle: &BattleState,
) -> Result<(), String> {
    let save_file = serialize_save_file::battle(user_id, quest_id, battle);
    provider.write_save_file(save_file).map_err(|errors| format_initialization_errors(&errors))
}

fn perform_action_internal<P: StateProvider + 'static>(
    provider: &P,
    request: &PerformActionRequest,
) -> bool {
    let metadata = request.metadata;
    let user_id = metadata.user_id;
    let request_id = metadata.request_id;

    // Check if we should process this action
    if let Some(last_response_version) = request.last_response_version {
        let stored_version = provider.get_last_response_version(user_id);
        if stored_version != Some(last_response_version) {
            warn!(
                ?user_id,
                ?last_response_version,
                ?stored_version,
                "Ignoring action: client is responding to an outdated response version"
            );
            return false;
        }
    }

    // Try to start processing - if we're already processing, ignore the action
    if !provider.start_processing(user_id) {
        warn!(?user_id, "Ignoring action: already processing another action for this user");
        return false;
    }

    let span =
        tracing::span!(Level::DEBUG, "perform_action", ?request_id, ?request.last_response_version);
    let _enter = span.enter();

    let result = catch_panic_conditionally(provider, || {
        let save_file_id = request.save_file_id.unwrap_or(user_id);
        let save = match provider.read_save_file(save_file_id) {
            Ok(Some(save)) => save,
            Ok(None) => {
                let error_msg = if request.save_file_id.is_some() {
                    format!("No save file found for request.save_file_id: {save_file_id:?}")
                } else {
                    format!("No save file found for user_id: {user_id:?}")
                };
                show_error_message(provider, user_id, None, error_msg);
                return;
            }
            Err(errors) => {
                show_error_message(provider, user_id, None, format_initialization_errors(&errors));
                return;
            }
        };

        let request_context = provider
            .get_request_context(user_id)
            .unwrap_or(RequestContext { logging_options: Default::default() });
        let Some((mut battle, quest_id)) =
            deserialize_save_file::battle(provider, &save, request_context)
        else {
            show_error_message(
                provider,
                user_id,
                None,
                format!("No battle found for save_file_id: {save_file_id:?}"),
            );
            return;
        };

        battle.animations = Some(AnimationData::default());
        handle_request_action(provider, request, user_id, &mut battle, request_id);

        if let Err(errors) =
            provider.write_save_file(serialize_save_file::battle(save_file_id, quest_id, &battle))
        {
            show_error_message(
                provider,
                user_id,
                Some(&battle),
                format!("Failed to save battle: {}", format_initialization_errors(&errors)),
            );
        }
    });

    if let Err(error) = result {
        show_error_message(provider, user_id, None, error);
    }

    true
}

fn format_initialization_errors(errors: &[InitializationError]) -> String {
    if errors.is_empty() {
        return "Unknown initialization error".to_string();
    }
    errors.iter().map(|e| e.format()).collect::<Vec<_>>().join("\n")
}

fn send_updates_to_user_and_opponent<P: StateProvider + 'static>(
    provider: &P,
    battle: &BattleState,
    user_id: UserId,
    player: PlayerName,
    request_context: &RequestContext,
    request_id: Option<Uuid>,
) {
    handle_battle_action::append_update(
        provider,
        user_id,
        renderer::connect(battle, user_id, provider.clone(), true),
        request_context,
        request_id,
        PollResponseType::Final,
    );

    if let PlayerType::User(opponent_id) = &battle.players.player(player.opponent()).player_type {
        handle_battle_action::append_update(
            provider,
            *opponent_id,
            renderer::connect(battle, *opponent_id, provider.clone(), true),
            request_context,
            request_id,
            PollResponseType::Final,
        );
    }
}

fn handle_request_action<P: StateProvider + 'static>(
    provider: &P,
    request: &PerformActionRequest,
    user_id: UserId,
    battle: &mut BattleState,
    request_id: Option<Uuid>,
) {
    let request_context = provider
        .get_request_context(user_id)
        .unwrap_or(RequestContext { logging_options: Default::default() });
    apply_battle_display_action::on_action_performed(provider.clone(), &request.action, user_id);

    match &request.action {
        GameAction::NoOp => {}
        GameAction::DebugAction(action) => {
            let player = renderer::player_name_for_user(&*battle, user_id);
            debug_actions::execute(provider, battle, user_id, player, action.clone());

            send_updates_to_user_and_opponent(
                provider,
                battle,
                user_id,
                player,
                &request_context,
                request_id,
            );
        }
        GameAction::BattleAction(action) => {
            let player = renderer::player_name_for_user(&*battle, user_id);
            handle_battle_action::execute(
                provider,
                battle,
                user_id,
                player,
                *action,
                &request_context,
                request_id,
            );
        }
        GameAction::BattleDisplayAction(action) => {
            let player = renderer::player_name_for_user(&*battle, user_id);
            let display_commands = apply_battle_display_action::execute(
                provider.clone(),
                action.clone(),
                player,
                user_id,
            );
            let mut connect_commands = renderer::connect(&*battle, user_id, provider.clone(), true);
            connect_commands.groups.extend(display_commands.groups);
            handle_battle_action::append_update(
                provider,
                user_id,
                connect_commands,
                &request_context,
                request_id,
                PollResponseType::Final,
            );
        }
        GameAction::Undo(player) => {
            if let Some(mut previous) = provider.pop_undo_entry(battle.id, *player) {
                previous.animations = Some(AnimationData::default());
                previous.tracing = Some(battle.tracing.clone().unwrap_or_default());
                *battle = previous;
                let player_name = renderer::player_name_for_user(&*battle, user_id);
                send_updates_to_user_and_opponent(
                    provider,
                    battle,
                    user_id,
                    player_name,
                    &request_context,
                    request_id,
                );
            } else {
                show_error_message(
                    provider,
                    user_id,
                    None,
                    "Failed to undo: Battle state not found.".to_string(),
                );
            }
        }
    };
}

/// Gets or creates a TestStateProvider for the given integration test ID.
/// This ensures that all requests for the same integration test use the same
/// provider instance.
fn get_test_state_provider(integration_test_id: Uuid) -> TestStateProvider {
    let mut providers = TEST_STATE_PROVIDERS.lock().unwrap();
    providers.entry(integration_test_id).or_default().clone()
}

fn show_error_message<P: StateProvider + 'static>(
    provider: &P,
    user_id: UserId,
    battle: Option<&BattleState>,
    error_message: String,
) {
    error!("Error in engine: {error_message}");

    if provider.should_panic_on_error() {
        panic!("Error in test: {error_message}");
    }

    let request_context = provider
        .get_request_context(user_id)
        .unwrap_or(RequestContext { logging_options: Default::default() });
    handle_battle_action::append_update(
        provider,
        user_id,
        error_message::display_error_message(battle, provider, error_message),
        &request_context,
        None,
        PollResponseType::Final,
    );
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

        let panic_msg = format!("{panic_info}");
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
                    let backtrace_str = format!("{backtrace:?}");
                    let filtered_backtrace = filter_backtrace(&backtrace_str);

                    format!(
                        "Error: {panic_msg} at {location}\n\nError details for developers:\n{info}\n{filtered_backtrace}"
                    )
                } else {
                    format!("Error: {panic_msg}\n\nNo backtrace available")
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

/// Conditionally catches panics based on the state provider.
/// In test mode (when provider.should_panic_on_error() returns true), panics
/// are propagated. Otherwise, panics are caught and converted to error
/// messages.
fn catch_panic_conditionally<F, T>(provider: &impl StateProvider, function: F) -> Result<T, String>
where
    F: FnOnce() -> T + panic::UnwindSafe,
{
    if provider.should_panic_on_error() { Ok(function()) } else { catch_panic(function) }
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
            writeln!(result, "{line}").ok();
        }
    }

    result
}
