use std::sync::{Arc, Condvar, Mutex};

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use battle_mutations::actions::apply_battle_action;
use battle_queries::battle_trace;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    ForPlayer, LegalActions, PrimaryLegalAction,
};
use battle_queries::macros::write_tracing_event;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::command::CommandSequence;
use display_data::request_data::PollResponseType;
use state_provider::state_provider::{PollResult, SpeculativeSearchState, StateProvider};
use tokio::task;
use tracing::{debug, instrument};
use uuid::Uuid;

pub fn poll(provider: &impl StateProvider, user_id: UserId) -> Option<PollResult> {
    provider.take_next_poll_result(user_id)
}

pub fn append_update(
    provider: &impl StateProvider,
    user_id: UserId,
    update: CommandSequence,
    context: &RequestContext,
    request_id: Option<Uuid>,
    response_type: PollResponseType,
) {
    write_tracing_event::write_commands(&update, context);
    provider.append_poll_result(user_id, PollResult {
        commands: update,
        request_id,
        response_type,
    });
}

#[instrument(skip_all, level = "debug")]
pub fn execute(
    provider: &(impl StateProvider + 'static),
    battle: &mut BattleState,
    initiated_by: UserId,
    player: PlayerName,
    action: BattleAction,
    context: &RequestContext,
    request_id: Option<Uuid>,
) {
    let mut current_player = player;
    let mut current_action = action;

    if should_push_undo_entry(current_action) {
        let mut snapshot = battle.clone();
        snapshot.animations = None;
        snapshot.tracing = None;
        provider.push_undo_entry(snapshot.id, current_player, snapshot);
    }

    loop {
        battle_trace!("Executing battle action", battle, current_action, request_id);
        apply_battle_action::execute(battle, current_player, current_action);

        let Some(next_player) = legal_actions::next_to_act(battle) else {
            battle_trace!("Rendering updates for game over", battle);
            render_updates(
                provider,
                battle,
                initiated_by,
                context,
                request_id,
                PollResponseType::Final,
            );
            return;
        };

        let legal_actions = legal_actions::compute(battle, next_player);
        if let Some(auto_action) = should_auto_execute_action(&legal_actions) {
            battle_trace!("Automatically executing action", battle, next_player, auto_action);
            current_player = next_player;
            current_action = auto_action;
            continue;
        }

        if let PlayerType::Agent(agent) = battle.players.player(next_player).player_type.clone() {
            battle_trace!("Rendering updates for AI player turn", battle);
            render_updates(
                provider,
                battle,
                initiated_by,
                context,
                request_id,
                PollResponseType::Incremental,
            );
            battle.animations = Some(AnimationData::default());
            battle_trace!("Selecting action for AI player", battle);

            if let Some(action) = get_speculative_response_action(provider, battle, action) {
                battle_trace!("[🌟] Speculative action hit", battle, action);
                current_action = action;
            } else {
                current_action = agent_search::select_action(battle, next_player, &agent);
            }

            current_player = next_player;
        } else {
            battle_trace!("Rendering updates for human player turn", battle);
            render_updates(
                provider,
                battle,
                initiated_by,
                context,
                request_id,
                PollResponseType::Final,
            );
            battle.animations = Some(AnimationData::default());
            let agent_player = next_player.opponent();

            if let PlayerType::Agent(agent) =
                &battle.players.player(agent_player).player_type.clone()
            {
                let legal = legal_actions::compute(battle, next_player);
                if let LegalActions::Standard { actions } = legal {
                    start_speculative_response_search(
                        provider,
                        battle,
                        agent_player,
                        agent,
                        next_player,
                        actions.primary,
                    );
                }
            }
            return;
        }
    }
}

pub fn should_auto_execute_action(legal_actions: &LegalActions) -> Option<BattleAction> {
    if legal_actions.len() == 1 {
        match legal_actions {
            LegalActions::Standard { .. }
                if legal_actions.contains(BattleAction::PassPriority, ForPlayer::Human) =>
            {
                Some(BattleAction::PassPriority)
            }
            LegalActions::Standard { .. }
                if legal_actions.contains(BattleAction::StartNextTurn, ForPlayer::Human) =>
            {
                Some(BattleAction::StartNextTurn)
            }
            LegalActions::SelectPromptChoicePrompt { choice_count: 1 } => {
                Some(BattleAction::SelectPromptChoice(0))
            }
            LegalActions::SelectCharacterPrompt { valid } if valid.len() == 1 => {
                valid.iter().next().map(BattleAction::SelectCharacterTarget)
            }
            LegalActions::SelectStackCardPrompt { valid } if valid.len() == 1 => {
                valid.iter().next().map(BattleAction::SelectStackCardTarget)
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Begins a speculative search for an agent action.
///
/// In order to optimistically improve performance, we assume that the human
/// player will respond to a battle state with their [PrimaryLegalAction]
/// (resolve, end turn, etc). While we are waiting for a user response, we start
/// computing a speculative response to this action in order to respond more
/// quickly if it is selected.
fn start_speculative_response_search(
    provider: &(impl StateProvider + 'static),
    battle: &mut BattleState,
    ai_player: PlayerName,
    agent: &GameAI,
    human_player: PlayerName,
    opponent_action: PrimaryLegalAction,
) {
    let assumed_action = match opponent_action {
        PrimaryLegalAction::PassPriority => BattleAction::PassPriority,
        PrimaryLegalAction::EndTurn => BattleAction::EndTurn,
        PrimaryLegalAction::StartNextTurn => BattleAction::StartNextTurn,
    };
    let mut simulation = battle.logical_clone();
    apply_battle_action::execute(&mut simulation, human_player, assumed_action);
    while let Some(next_player) = legal_actions::next_to_act(&simulation) {
        if let Some(auto) =
            should_auto_execute_action(&legal_actions::compute(&simulation, next_player))
        {
            apply_battle_action::execute(&mut simulation, next_player, auto);
            continue;
        }
        break;
    }
    if legal_actions::next_to_act(&simulation) != Some(ai_player) {
        battle_trace!(
            "[🔮] Skipping speculation, ai_player is not next to act",
            battle,
            opponent_action
        );
        return;
    }
    let result = Arc::new((Mutex::new(None), Condvar::new()));
    let result_clone = result.clone();
    let agent_clone = *agent;
    battle_trace!("[🔮] Starting speculative action search", battle, opponent_action);

    task::spawn_blocking(move || {
        let action = agent_search::select_action(&simulation, ai_player, &agent_clone);
        if let Ok(mut guard) = result_clone.0.lock() {
            *guard = Some(action);
            result_clone.1.notify_all();
        }
    });
    provider.set_speculative_search(battle.id, SpeculativeSearchState { assumed_action, result });
}

/// Returns the computed speculative response action.
///
/// If the provided `action` matches the action we assumed the user would take
/// when we started our speculative response search, this returns a computed
/// agent response to that action. If the user took a different action, this
/// returns None.
///
/// If the AI evaluation has not yet completed, but the `action` here matches
/// the assumed action, this blocks until the evaluation is complete and returns
/// its value.
fn get_speculative_response_action(
    provider: &(impl StateProvider + 'static),
    battle: &BattleState,
    action: BattleAction,
) -> Option<BattleAction> {
    let search = provider.take_speculative_search(battle.id)?;
    if search.assumed_action != action {
        let expected = search.assumed_action;
        debug!(?action, ?expected, "[👿] Speculative Action miss");
        return None;
    }
    let (lock, cvar) = &*search.result;
    let mut guard = lock.lock().unwrap();
    while guard.is_none() {
        guard = cvar.wait(guard).unwrap();
    }
    *guard
}

fn should_push_undo_entry(action: BattleAction) -> bool {
    !matches!(
        action,
        BattleAction::SelectOrderForDeckCard(..)
            | BattleAction::SelectVoidCardTarget(..)
            | BattleAction::SelectHandCardTarget(..)
            | BattleAction::SelectModalEffectChoice(..)
    )
}

fn render_updates(
    provider: &(impl StateProvider + 'static),
    battle: &BattleState,
    user_id: UserId,
    context: &RequestContext,
    request_id: Option<Uuid>,
    response_type: PollResponseType,
) {
    let player_name = renderer::player_name_for_user(battle, user_id);
    let player_updates = renderer::render_updates(battle, user_id, (*provider).clone());
    append_update(provider, user_id, player_updates, context, request_id, response_type);

    if let PlayerType::User(opponent_id) =
        &battle.players.player(player_name.opponent()).player_type
    {
        let opponent_updates = renderer::render_updates(battle, *opponent_id, (*provider).clone());
        append_update(provider, *opponent_id, opponent_updates, context, request_id, response_type);
    }
}
