use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use actions::battle_actions;
use ai_agents_old::agent_search;
use battle_data_old::actions::battle_action_data::BattleAction;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_animations::animation_data::AnimationData;
use battle_data_old::battle_player::player_data::PlayerType;
use battle_queries_old::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use display_old::rendering::renderer;
use display_data_old::command::CommandSequence;
use logging::battle_trace;
use tracing::instrument;

static PENDING_UPDATES: LazyLock<Mutex<HashMap<UserId, Vec<CommandSequence>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[instrument(name = "handle_battle_action", level = "debug", skip(battle))]
pub fn execute(
    battle: &mut BattleData,
    initiated_by: UserId,
    player: PlayerName,
    action: BattleAction,
) {
    let mut current_player = player;
    let mut current_action = action;

    loop {
        battle_trace!("Executing battle action", battle, current_action);
        battle_actions::execute(battle, current_player, current_action);

        let Some(next_player) = legal_actions::next_to_act(battle) else {
            battle_trace!("Rendering updates for game over", battle);
            render_updates(battle, initiated_by);
            return;
        };

        let legal_actions = legal_actions::compute(battle, next_player, LegalActions::default());
        if legal_actions == vec![BattleAction::PassPriority] {
            battle_trace!("Automatically executing PassPriority", battle, next_player);
            current_player = next_player;
            current_action = BattleAction::PassPriority;
            continue;
        }

        if legal_actions == vec![BattleAction::StartNextTurn] {
            battle_trace!("Automatically executing StartNextTurn", battle, next_player);
            current_player = next_player;
            current_action = BattleAction::StartNextTurn;
            continue;
        }

        if let PlayerType::Agent(agent) = battle.player(next_player).player_type.clone() {
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
    let mut updates = PENDING_UPDATES.lock().unwrap();
    updates.entry(user_id).or_default().push(update);
}

fn render_updates(battle: &BattleData, user_id: UserId) {
    let player_name = renderer::player_name_for_user(battle, user_id);
    let player_updates = renderer::render_updates(battle, user_id);
    let mut updates = PENDING_UPDATES.lock().unwrap();
    updates.entry(user_id).or_default().push(player_updates);

    if let PlayerType::User(opponent_id) = &battle.player(player_name.opponent()).player_type {
        let opponent_updates = renderer::render_updates(battle, *opponent_id);
        updates.entry(*opponent_id).or_default().push(opponent_updates);
    }
}
