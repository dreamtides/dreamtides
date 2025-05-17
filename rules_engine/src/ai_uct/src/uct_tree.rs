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
    pub tried: Vec<BattleAction>,
}

#[derive(Debug, Clone)]
pub struct UctSearchResult {
    /// Action the agent wants to take
    pub action: BattleAction,

    /// Graph with information to reuse for future searches.
    pub next_graph: SearchGraph,

    /// Root node of the returned graph, with outbound edges corresponding to
    /// possible actions in the game state after applying `action`.
    ///
    /// Note that the returned graph will not be usable for a search as-is if
    /// the next player to act is not the agent. A subgraph will need to be
    /// extracted once it is again the agent's turn.
    pub next_root: NodeIndex,
}

/// Operation mode for child scoring.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SelectionMode {
    /// Balance between trying new children and re-visiting existing children.
    Exploration,
    /// Select the best overall child without giving any weight to exploration.
    RewardOnly,
}
