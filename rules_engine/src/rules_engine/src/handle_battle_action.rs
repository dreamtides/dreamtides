use action_data::battle_action::BattleAction;
use actions::battle_actions;
use ai_agents::agent_search;
use battle_data::battle::battle_data::BattleData;
use battle_queries::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::command::CommandSequence;
use tracing::{info, instrument};

#[instrument(name = "handle_battle_action", level = "debug", skip(battle))]
pub fn execute(
    battle: &mut BattleData,
    player: PlayerName,
    action: BattleAction,
) -> CommandSequence {
    info!("Executing battle action: {:?}", action);
    battle_actions::execute(battle, player, action);

    let Some(next_player) = legal_actions::next_to_act(battle) else {
        info!("Rendering updates for game over");
        return renderer::render_updates(battle);
    };

    // Check if the only legal action is ResolveStack and automatically execute it
    let legal_actions = legal_actions::compute(battle, next_player, LegalActions::default());
    if legal_actions == vec![BattleAction::ResolveStack] {
        info!("Automatically executing ResolveStack");
        return execute(battle, next_player, BattleAction::ResolveStack);
    }

    if let Some(agent) = battle.player(next_player).ai.as_ref() {
        info!("Selecting action for AI player");
        let next_action = agent_search::select_action(battle, next_player, agent);
        info!("Executing action for AI player: {:?}", next_action);
        execute(battle, next_player, next_action)
    } else {
        info!("Rendering updates for human player");
        renderer::render_updates(battle)
    }
}
