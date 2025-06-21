use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use ai_agents::agent_search;
use battle_mutations::actions::apply_battle_action;
use battle_queries::battle_trace;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
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
use tracing::instrument;
use uuid::Uuid;

use crate::engine::PollResult;

static PENDING_UPDATES: LazyLock<Mutex<HashMap<UserId, Vec<PollResult>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn poll(user_id: UserId) -> Option<PollResult> {
    let mut updates = PENDING_UPDATES.lock().unwrap();
    if let Some(user_updates) = updates.get_mut(&user_id) {
        if !user_updates.is_empty() {
            return Some(user_updates.remove(0));
        }
    }
    None
}

pub fn append_update(
    user_id: UserId,
    update: CommandSequence,
    context: &RequestContext,
    request_id: Option<Uuid>,
    response_type: PollResponseType,
) {
    write_tracing_event::write_commands(&update, context);
    let mut updates = PENDING_UPDATES.lock().unwrap();
    updates.entry(user_id).or_default().push(PollResult {
        commands: update,
        request_id,
        response_type,
    });
}

#[instrument(name = "handle_battle_action", level = "debug", skip(battle, context))]
pub fn execute(
    battle: &mut BattleState,
    initiated_by: UserId,
    player: PlayerName,
    action: BattleAction,
    context: &RequestContext,
    request_id: Option<Uuid>,
) {
    let mut current_player = player;
    let mut current_action = action;

    loop {
        battle_trace!("Executing battle action", battle, current_action, request_id);
        apply_battle_action::execute(battle, current_player, current_action);

        let Some(next_player) = legal_actions::next_to_act(battle) else {
            battle_trace!("Rendering updates for game over", battle);
            render_updates(battle, initiated_by, context, request_id, PollResponseType::Final);
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
                battle,
                initiated_by,
                context,
                request_id,
                PollResponseType::Incremental,
            );
            battle.animations = Some(AnimationData::default());

            battle_trace!("Selecting action for AI player", battle);
            current_action = agent_search::select_action(battle, next_player, &agent);
            current_player = next_player;
        } else {
            battle_trace!("Rendering updates for human player turn", battle);
            render_updates(battle, initiated_by, context, request_id, PollResponseType::Final);
            battle.animations = Some(AnimationData::default());
            return;
        }
    }
}

pub fn should_auto_execute_action(legal_actions: &LegalActions) -> Option<BattleAction> {
    if legal_actions.len() == 1 {
        match legal_actions {
            LegalActions::Standard { .. } if legal_actions.contains(BattleAction::PassPriority) => {
                Some(BattleAction::PassPriority)
            }
            LegalActions::Standard { .. }
                if legal_actions.contains(BattleAction::StartNextTurn) =>
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

fn render_updates(
    battle: &BattleState,
    user_id: UserId,
    context: &RequestContext,
    request_id: Option<Uuid>,
    response_type: PollResponseType,
) {
    let player_name = renderer::player_name_for_user(battle, user_id);
    let player_updates = renderer::render_updates(battle, user_id);
    append_update(user_id, player_updates, context, request_id, response_type);

    if let PlayerType::User(opponent_id) =
        &battle.players.player(player_name.opponent()).player_type
    {
        let opponent_updates = renderer::render_updates(battle, *opponent_id);
        append_update(*opponent_id, opponent_updates, context, request_id, response_type);
    }
}
