use battle_state::actions::battle_actions::BattleAction;
use core_data::types::PlayerName;
use ego_tree::Tree;

pub struct UctTree {
    tree: Tree<SearchNode>,
}

pub struct SearchNode {
    /// Action taken to create this node
    pub action: BattleAction,
    /// Player who acted to create this node
    pub player: PlayerName,
    /// Q(v): Total reward of all playouts that passed through this state
    pub total_reward: f64,
    /// N(v): Visit count for this node
    pub visit_count: u32,
}
