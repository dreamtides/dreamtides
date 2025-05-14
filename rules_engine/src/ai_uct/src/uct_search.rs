use std::f64::consts;

use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use petgraph::prelude::NodeIndex;

use crate::uct_config::UctConfig;
use crate::uct_tree::{SearchGraph, SelectionMode, UctSearchResult};

/// Monte Carlo search algorithm.
///
/// Searches for an action for `player` to take in the given `battle` state. The
/// provided `graph` and `root` should correspond to a search graph rooted at
/// this state, i.e. one where the agent's possible actions form outgoing edges.
/// When starting, it's fine to provide an empty graph with a default root node.
///
/// Returns a [UctSearchResult] with an action to perform as well as graph data
/// to reuse in future searches.
///
/// Monte carlo tree search operates over a tree of game state nodes
/// connected by game actions. The search follows these three steps
/// repeatedly:
///
/// 1) **Tree Policy:** Find a node in the tree which has not previously been
///    explored. The UCT algorithm is one mathematical heuristic for how to
///    prioritize nodes to explore.
///
/// 2) **Default Policy:** Score this node to determine its reward value (∆),
///    typically by playing random moves until the game terminates.
///
/// 3) **Backpropagation:** Walk back up the tree, adding the resulting reward
///    value to each parent node.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 UCTSEARCH(s₀)
///   create root node v₀ with state s₀
///   𝐰𝐡𝐢𝐥𝐞 within computational budget 𝐝𝐨
///     v₁ ← TREEPOLICY(v₀)
///     ∆ ← DEFAULTPOLICY(s(v₁))
///     BACKUP(v₁, ∆)
///   𝐫𝐞𝐭𝐮𝐫𝐧 𝒂(BESTCHILD(v₀, 0))
/// ```
pub fn search(
    battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
    graph: &SearchGraph,
    root: NodeIndex,
) -> UctSearchResult {
    todo!("Implement UCT search")
}

/// Returns a descendant node to evaluate next for the provided parent node.
///
/// This 'tree policy' function returns either:
///  * A node which has not yet been explored
///  * The best terminal node descendant, if all nodes have been explored.
///
/// If possible actions are available from this node which have not yet been
/// explored, selects an action and applies it, returning the result as a
/// new child. Otherwise, selects the best child to explore based on
/// visit counts and known rewards, using the `best_child` algorithm,
/// and then repeats this process recursively until an unseen node is
/// found (or the best child is terminal).
///
/// Mutates the provided [BattleState] to represent the game state at the
/// returned node.
///
/// Cᵖ is the exploration constant, Cᵖ = 1/√2 was suggested by Kocsis and
/// Szepesvári as a good choice.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 TREEPOLICY(v)
///   𝐰𝐡𝐢𝐥𝐞 v is nonterminal 𝐝𝐨
///     𝐢𝐟 v not fully expanded 𝐭𝐡𝐞𝐧
///       𝐫𝐞𝐭𝐮𝐫𝐧 EXPAND(v)
///     𝐞𝐥𝐬𝐞
///       v ← BESTCHILD(v, Cᵖ)
///   𝐫𝐞𝐭𝐮𝐫𝐧 v
/// ```
fn next_evaluation_target(
    battle: &mut BattleState,
    graph: &mut SearchGraph,
    from_node: NodeIndex,
) -> NodeIndex {
    todo!("Implement UCT search")
}

/// Adds a new node to the search graph, the 'expand' function.
///
/// Generates a new tree node by applying the next untried action from the
/// provided input node. Mutates the provided [GameState] to apply the
/// provided game action.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 EXPAND(v)
///   choose 𝒂 ∈ untried actions from A(s(v))
///   add a new child v′ to v
///     with s(v′) = f(s(v), 𝒂)
///     and 𝒂(v′) = 𝒂
///   𝐫𝐞𝐭𝐮𝐫𝐧 v′
/// ```
fn add_node(
    battle: &mut BattleState,
    graph: &mut SearchGraph,
    player: PlayerName,
    from_node: NodeIndex,
    action: BattleAction,
) -> NodeIndex {
    todo!("Implement UCT search")
}

struct BestChild {
    action: BattleAction,
    node: NodeIndex,
}

/// Picks the most promising child node to explore next.
///
/// This picks a child based on its computed reward and visit count, subject to
/// the requested [SelectionMode]. Returns the action taken to produce this
/// child and its associated node.
fn best_child(
    graph: &SearchGraph,
    from_node: NodeIndex,
    legal: &LegalActions,
    selection_mode: SelectionMode,
) -> BestChild {
    todo!("Implement UCT search")
}

/// Walks back up the search tree, adding the resulting reward value to each
/// parent node in sequence. Rewards are negated when not earned for the
/// `maximizing_player`.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 BACKUP(v,∆)
///   𝐰𝐡𝐢𝐥𝐞 v is not null 𝐝𝐨
///     N(v) ← N(v) + 1
///     Q(v) ← Q(v) + ∆(v, p)
///     v ← parent of v
/// ```
fn back_propagate_rewards(
    graph: &mut SearchGraph,
    maximizing_player: PlayerName,
    leaf_node: NodeIndex,
    reward: f64,
) {
    todo!("Implement UCT search")
}

/// Computes the score for a child node based on its parent's visit count and
/// active [SelectionMode].
///
/// This implements the UCT1 algorithm for child scoring, a standard approach
/// for selecting children and solution to the 'multi-armed bandit' problem.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 BESTCHILD(v,c)
///   𝐫𝐞𝐭𝐮𝐫𝐧 argmax(
///     v′ ∈ children of v:
///     Q(v′) / N(v′) +
///     c * √ [ 2 * ln(N(v)) / N(v′) ]
///   )
/// ```
fn child_score(
    parent_visits: u32,
    child_visits: u32,
    reward: f64,
    selection_mode: SelectionMode,
) -> f64 {
    let exploitation = reward / f64::from(child_visits);
    let exploration =
        f64::sqrt((2.0 * f64::ln(f64::from(parent_visits))) / f64::from(child_visits));
    let exploration_bias = match selection_mode {
        SelectionMode::Exploration => consts::FRAC_1_SQRT_2,
        SelectionMode::Best => 0.0,
    };
    exploitation + (exploration_bias * exploration)
}
