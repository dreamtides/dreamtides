use battle_state::actions::battle_actions::BattleAction;
use core_data::types::PlayerName;
use ordered_float::OrderedFloat;
use petgraph::prelude::NodeIndex;
use petgraph::Graph;

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

#[derive(Debug, Clone)]
pub struct UctSearchResult {
    /// Action the agent wants to take
    pub action: BattleAction,

    /// Graph with information to reuse for future searches.
    /// 
    /// Outbound edges correspond to possible actions in the game state after
    /// applying `action`.
    ///
    /// Note that the returned graph will not be usable for a search as-is if
    /// the next player to act is not the agent. A subgraph will need to be
    /// extracted once it is again the agent's turn.
    pub next_graph: Option<GraphWithRoot>,
}

/// Operation mode for child scoring.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SelectionMode {
    /// Balance between trying new children and re-visiting existing children.
    Exploration,
    /// Select the best overall child without giving any weight to exploration.
    RewardOnly,
}
