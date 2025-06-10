use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use ai_agents::agent_search;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::command::CommandSequence;
use tracing::instrument;
use tracing_macros::{battle_trace, write_tracing_event};

static PENDING_UPDATES: LazyLock<Mutex<HashMap<UserId, Vec<CommandSequence>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn poll(user_id: UserId) -> Option<CommandSequence> {
    let mut updates = PENDING_UPDATES.lock().unwrap();
    if let Some(user_updates) = updates.get_mut(&user_id) {
        if !user_updates.is_empty() {
            return Some(user_updates.remove(0));
        }
    }
    None
}

/// Appends an update to the pending updates for a user.
pub fn append_update(user_id: UserId, update: CommandSequence) {
    write_tracing_event::write_commands(&update, &context);
    let mut updates = PENDING_UPDATES.lock().unwrap();
    updates.entry(user_id).or_default().push(update);
}

#[instrument(name = "handle_battle_action", level = "debug", skip(battle))]
pub fn execute(
    battle: &mut BattleState,
    initiated_by: UserId,
    player: PlayerName,
    action: BattleAction,
) {
    let mut current_player = player;
    let mut current_action = action;

    loop {
        battle_trace!("Executing battle action", battle, current_action);
        apply_battle_action::execute(battle, current_player, current_action);

        let Some(next_player) = legal_actions::next_to_act(battle) else {
            battle_trace!("Rendering updates for game over", battle);
            render_updates(battle, initiated_by);
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
            render_updates(battle, initiated_by);
            battle.animations = Some(AnimationData::default());

            battle_trace!("Selecting action for AI player", battle);
            current_action = agent_search::select_action(battle, next_player, &agent);
            current_player = next_player;
        } else {
            battle_trace!("Rendering updates for human player turn", battle);
            render_updates(battle, initiated_by);
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

fn render_updates(battle: &BattleState, user_id: UserId) {
    let player_name = renderer::player_name_for_user(battle, user_id);
    let player_updates = renderer::render_updates(battle, user_id);
    let mut updates = PENDING_UPDATES.lock().unwrap();
    updates.entry(user_id).or_default().push(player_updates);

    if let PlayerType::User(opponent_id) =
        &battle.players.player(player_name.opponent()).player_type
    {
        let opponent_updates = renderer::render_updates(battle, *opponent_id);
        updates.entry(*opponent_id).or_default().push(opponent_updates);
    }
}
