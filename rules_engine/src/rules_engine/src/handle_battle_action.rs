use action_data::battle_action::BattleAction;
use actions::battle_actions;
use agents::agent_search;
use battle_data::battle::battle_data::BattleData;
use battle_queries::legal_action_queries::legal_actions;
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::command::CommandSequence;
use tracing::instrument;

#[instrument(name = "actions_execute", level = "debug", skip(battle))]
pub fn execute(
    battle: &mut BattleData,
    player: PlayerName,
    action: BattleAction,
) -> CommandSequence {
    let mut next_action = action;
    let mut current_player = player;
    loop {
        battle_actions::execute(battle, current_player, next_action);
        current_player = legal_actions::next_to_act(battle);
        if current_player == PlayerName::User {
            return renderer::render_updates(battle);
        } else {
            next_action = agent_search::select_action(battle, current_player);
        }
    }
}
