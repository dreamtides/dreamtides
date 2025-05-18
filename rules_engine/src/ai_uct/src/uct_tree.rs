use battle_state::actions::battle_actions::BattleAction;
use core_data::types::PlayerName;
use ordered_float::OrderedFloat;
use petgraph::prelude::NodeIndex;
use petgraph::Graph;

/// Tree data structure to store monte carlo search results.
///
/// I've tried this with other representations such as the 'ego-tree' crate and
/// other graph data structures in 'petgraph' like GraphMap, but none of them
/// have notably better performance.
pub type SearchGraph = Graph<SearchNode, SearchEdge>;

#[derive(Debug, Clone, Copy)]
pub struct SearchEdge {
    /// Action taken to create the next node
    pub action: BattleAction,
}

#[derive(Debug, Clone)]
pub struct SearchNode {
    /// Player who acted to create this node
    pub player: PlayerName,
    /// Q(v): Total reward of all playouts that passed through this state
    pub total_reward: OrderedFloat<f64>,
    /// N(v): Visit count for this node
    pub visit_count: u32,
    /// Actions we have already tried in this state.
    ///
    /// Vec has generally outperformed various set & bitset data structures
    /// here.
    pub tried: Vec<BattleAction>,
}

/// A search graph with its root node
#[derive(Debug, Clone)]
pub struct GraphWithRoot {
    pub graph: SearchGraph,
    pub root: NodeIndex,
}

/// Operation mode for child scoring.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SelectionMode {
    /// Balance between trying new children and re-visiting existing children.
    Exploration,
    /// Select the best overall child without giving any weight to exploration.
    RewardOnly,
}
