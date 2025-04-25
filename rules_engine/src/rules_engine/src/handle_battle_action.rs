use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use action_data::battle_action::BattleAction;
use actions::battle_actions;
use ai_agents::agent_search;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use battle_data::battle_player::player_data::PlayerType;
use battle_queries::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::command::CommandSequence;
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
) -> CommandSequence {
    let mut current_player = player;
    let mut current_action = action;

    loop {
        battle_trace!("Executing battle action", battle, current_action);
        battle_actions::execute(battle, current_player, current_action);

        let Some(next_player) = legal_actions::next_to_act(battle) else {
            battle_trace!("Rendering updates for game over", battle);
            return render_updates(battle, initiated_by);
        };

        // Check if the only legal action is ResolveStack and automatically execute it
        let legal_actions = legal_actions::compute(battle, next_player, LegalActions::default());
        if legal_actions == vec![BattleAction::ResolveStack] {
            battle_trace!("Automatically executing ResolveStack", battle);
            current_player = next_player;
            current_action = BattleAction::ResolveStack;
            continue;
        }

        if let PlayerType::Agent(agent) = battle.player(next_player).player_type.clone() {
            battle_trace!("Selecting action for AI player", battle);
            let next_action = agent_search::select_action(battle, next_player, &agent);
            battle_trace!("Executing action for AI player", battle, next_action);
            current_player = next_player;
            current_action = next_action;
            continue;
        } else {
            battle_trace!("Rendering updates for player", battle);
            let result = render_updates(battle, initiated_by);
            battle.animations = Some(AnimationData::default());
            return result;
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

fn render_updates(battle: &BattleData, user_id: UserId) -> CommandSequence {
    let player_name = renderer::player_name_for_user(battle, user_id);
    let player_updates = renderer::render_updates(battle, user_id);

    if let PlayerType::User(opponent_id) = &battle.player(player_name.opponent()).player_type {
        let opponent_updates = renderer::render_updates(battle, *opponent_id);
        let mut updates = PENDING_UPDATES.lock().unwrap();
        updates.entry(*opponent_id).or_default().push(opponent_updates);
    }

    player_updates
}
