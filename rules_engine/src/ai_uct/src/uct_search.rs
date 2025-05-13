use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use ego_tree::NodeRef;

use crate::uct_tree::SearchNode;

fn tree_policy(battle: &mut BattleState, node: NodeRef<SearchNode>) {
    while let Some(current_player) = legal_actions::next_to_act(battle) {
        if let Some(unvisited) = node.children().find(|child| !child.has_children()) {
            // An action exists which has not yet been tried
            expand(battle, current_player, unvisited);
        }
    }
}

fn expand(battle: &mut BattleState, player: PlayerName, node: NodeRef<SearchNode>) {
    assert!(!node.has_children(), "Expected node without children in expand");
    apply_battle_action::execute(battle, player, node.value().action);
}
