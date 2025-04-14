use action_data::battle_action::BattleAction;
use actions::battle_actions;
use agents::agent_search;
use battle_data::battle::battle_data::BattleData;
use battle_queries::legal_action_queries::legal_actions;
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::command::CommandSequence;
use tracing::instrument;

#[instrument(name = "handle_battle_action", level = "debug", skip(battle))]
pub fn execute(
    battle: &mut BattleData,
    player: PlayerName,
    action: BattleAction,
) -> CommandSequence {
    battle_actions::execute(battle, player, action);
    let Some(next_player) = legal_actions::next_to_act(battle) else {
        // Game over.
        return renderer::render_updates(battle);
    };
    if let Some(agent) = battle.player(next_player).agent.as_ref() {
        let next_action = agent_search::select_action(battle, next_player, agent);
        execute(battle, next_player, next_action)
    } else {
        // Return response for human player.
        renderer::render_updates(battle)
    }
}
