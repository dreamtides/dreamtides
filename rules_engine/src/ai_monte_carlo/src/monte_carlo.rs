//! An implementation of Monte Carlo Tree Search.
//!
//! This implementation is based on the pseudocode given in "A Survey of Monte
//! Carlo Tree Search Methods" by Browne et al. in IEEE Transactions on
//! Computational Intelligence and AI in Games, Vol. 4, No. 1, March 2012.

use std::cmp;
use std::collections::HashSet;
use std::time::Instant;

use ai_core::agent::AgentConfig;
use ai_core::game_state_node::{GameStateNode, GameStatus};
use ai_core::selection_algorithm::SelectionAlgorithm;
use ai_core::state_evaluator::StateEvaluator;
use petgraph::prelude::{EdgeRef, NodeIndex};
use petgraph::{Direction, Graph};
use rand::seq::IteratorRandom;
use tracing::{debug, info};

use crate::child_score::{ChildScoreAlgorithm, SelectionMode};

/// Plays out a game using random moves until a terminal state is reached.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 DEFAULTPOLICY(s)
///   𝐰𝐡𝐢𝐥𝐞 s is non-terminal 𝐝𝐨
///     choose 𝒂 ∈ A(s) uniformly at random
///     s ← f(s,𝒂)
///   𝐫𝐞𝐭𝐮𝐫𝐧 reward for state s
/// ```
pub struct RandomPlayoutEvaluator {}

impl<TState: GameStateNode> StateEvaluator<TState> for RandomPlayoutEvaluator {
    fn evaluate(&self, input: &TState, player: TState::PlayerName) -> i32 {
        let mut game = input.make_copy();
        loop {
            match game.status() {
                GameStatus::Completed { winner } => {
                    return if winner == player { 1 } else { -1 };
                }
                GameStatus::InProgress { current_turn } => {
                    let action = game
                        .legal_actions(current_turn)
                        .choose(&mut rand::rng())
                        .expect("No actions found");
                    game.execute_action(current_turn, action);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct SearchNode<TState: GameStateNode> {
    /// Player who acted to create this node
    pub player: TState::PlayerName,
    /// Q(v): Total reward of all playouts that passed through this state
    pub total_reward: f64,
    /// N(v): Visit count for this node
    pub visit_count: u32,
}

struct SearchEdge<TState: GameStateNode> {
    pub action: TState::Action,
}

type SearchGraph<TState> = Graph<SearchNode<TState>, SearchEdge<TState>>;

/// Monte Carlo search algorithm.
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
pub struct MonteCarloAlgorithm<TScoreAlgorithm: ChildScoreAlgorithm> {
    pub child_score_algorithm: TScoreAlgorithm,
    pub max_iterations: Option<u32>,
}

impl<TScoreAlgorithm: ChildScoreAlgorithm> SelectionAlgorithm
    for MonteCarloAlgorithm<TScoreAlgorithm>
{
    fn pick_action<TStateNode, TEvaluator>(
        &self,
        config: AgentConfig,
        node: &TStateNode,
        evaluator: &TEvaluator,
        player: TStateNode::PlayerName,
    ) -> TStateNode::Action
    where
        TStateNode: GameStateNode,
        TEvaluator: StateEvaluator<TStateNode>,
    {
        self.run_search(
            |i| {
                i % 100 == 0 && config.deadline < Instant::now()
                    || self.max_iterations.is_some_and(|max| i >= max)
            },
            node,
            evaluator,
            player,
        )
    }
}

impl<TScoreAlgorithm: ChildScoreAlgorithm> MonteCarloAlgorithm<TScoreAlgorithm> {
    pub fn run_search<TStateNode: GameStateNode, TEvaluator: StateEvaluator<TStateNode>>(
        &self,
        should_halt: impl Fn(u32) -> bool,
        node: &TStateNode,
        evaluator: &TEvaluator,
        player: TStateNode::PlayerName,
    ) -> TStateNode::Action {
        let mut graph = SearchGraph::new();
        let root = graph.add_node(SearchNode { total_reward: 0.0, visit_count: 1, player });
        let legal = node.legal_actions(player).collect::<HashSet<_>>();
        assert!(legal.len() > 1, "Expected two or more legal actions");

        let mut i = 0;
        while !should_halt(i) {
            let mut game = node.make_randomized_copy(player);
            let node = self.tree_policy(&mut graph, &mut game, root);
            let reward = f64::from(evaluator.evaluate(&game, player));
            Self::backup(&mut graph, player, node, reward);
            i += 1;
        }

        info!("Halting monte carlo search after {} iterations", i);
        self.log_results(&graph, root, &legal);

        let (action, _) = self.best_child(&graph, root, &legal, SelectionMode::Best);
        action
    }

    /// Returns a descendant node to examine next for the provided parent node,
    /// either:
    ///  * A node which has not yet been explored
    ///  * The best terminal node descendant, if all nodes have been explored.
    ///
    /// If possible actions are available from this node which have not yet been
    /// explored, selects an action and applies it, returning the result as a
    /// new child. Otherwise, selects the best child to explore based on
    /// visit counts and known rewards, using the [best_child] algorithm,
    /// and then repeats this process recursively until an unseen node is
    /// found (or the best child is terminal).
    ///
    /// Mutates the provided [GameState] to represent the game state at the
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
    fn tree_policy<TState: GameStateNode>(
        &self,
        graph: &mut SearchGraph<TState>,
        game: &mut TState,
        mut node: NodeIndex,
    ) -> NodeIndex {
        while let GameStatus::InProgress { current_turn } = game.status() {
            let actions = game.legal_actions(current_turn).collect::<HashSet<_>>();
            let explored = graph.edges(node).map(|e| e.weight().action).collect::<HashSet<_>>();
            if let Some(action) = actions.iter().find(|a| !explored.contains(a)) {
                // An action exists which has not yet been tried
                return Self::expand(graph, game, current_turn, node, *action);
            } else {
                // All actions have been tried, recursively search the best candidate
                let (action, best) =
                    self.best_child(graph, node, &actions, SelectionMode::Exploration);
                game.execute_action(current_turn, action);
                node = best;
            }
        }
        node
    }

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
    fn expand<TState: GameStateNode>(
        graph: &mut SearchGraph<TState>,
        game: &mut TState,
        player: TState::PlayerName,
        source: NodeIndex,
        action: TState::Action,
    ) -> NodeIndex {
        game.execute_action(player, action);
        let target = graph.add_node(SearchNode { player, total_reward: 0.0, visit_count: 0 });
        graph.add_edge(source, target, SearchEdge { action });
        target
    }

    /// Picks the most promising child node to explore, returning its associated
    /// action and node identifier.
    ///
    /// This implementation is using the UCT1 algorithm, a standard approach for
    /// selecting children and solution to the 'multi-armed bandit' problem.
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
    fn best_child<TState: GameStateNode>(
        &self,
        graph: &SearchGraph<TState>,
        node: NodeIndex,
        legal: &HashSet<TState::Action>,
        selection_mode: SelectionMode,
    ) -> (TState::Action, NodeIndex) {
        let parent_visits = graph[node].visit_count;
        let (edge, _) = graph
            .edges(node)
            // We re-check action legality here because the set of legal actions can change between
            // visits, e.g. if different cards are drawn
            .filter(|edge| legal.contains(&edge.weight().action))
            .map(|edge| {
                let child = &graph[edge.target()];
                // This can technically panic when invoked from root with a very small
                // simulation count, so don't do that :)
                assert_ne!(child.visit_count, 0);
                (
                    edge,
                    self.child_score_algorithm.score(
                        f64::from(parent_visits),
                        f64::from(child.visit_count),
                        child.total_reward,
                        selection_mode,
                    ),
                )
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .expect("No children found");
        (edge.weight().action, edge.target())
    }

    /// Once a playout is completed, the backpropagation step walks back up the
    /// hierarchy of parent nodes, adding the resulting reward value to each
    /// one.
    ///
    /// Pseudocode:
    /// ```text
    /// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 BACKUP(v,∆)
    ///   𝐰𝐡𝐢𝐥𝐞 v is not null 𝐝𝐨
    ///     N(v) ← N(v) + 1
    ///     Q(v) ← Q(v) + ∆(v, p)
    ///     v ← parent of v
    /// ```
    fn backup<TState: GameStateNode>(
        graph: &mut SearchGraph<TState>,
        maximizing_player: TState::PlayerName,
        mut node: NodeIndex,
        reward: f64,
    ) {
        loop {
            let weight = graph.node_weight_mut(node).expect("Node not found");
            weight.visit_count += 1;
            weight.total_reward +=
                if weight.player == maximizing_player { reward } else { -reward };

            node = match graph.neighbors_directed(node, Direction::Incoming).next() {
                Some(n) => n,
                _ => break,
            };
        }
    }

    fn log_results<TState: GameStateNode>(
        &self,
        graph: &SearchGraph<TState>,
        node: NodeIndex,
        legal: &HashSet<TState::Action>,
    ) {
        let parent_visits = graph[node].visit_count;

        let mut scored_actions = graph
            .edges(node)
            .map(|edge| {
                let child = &graph[edge.target()];
                assert_ne!(child.visit_count, 0);
                (
                    edge.weight().action,
                    self.child_score_algorithm.score(
                        f64::from(parent_visits),
                        f64::from(child.visit_count),
                        child.total_reward,
                        SelectionMode::Best,
                    ),
                    child.visit_count,
                    child.total_reward,
                )
            })
            .collect::<Vec<_>>();

        scored_actions.sort_by(|(_, a_score, _, _), (_, b_score, _, _)| {
            b_score.partial_cmp(a_score).unwrap()
        });

        let display_count = cmp::min(3, scored_actions.len());
        if display_count > 0 {
            debug!("Top {} actions of {}:", display_count, scored_actions.len());
            for (i, (action, score, visits, reward)) in
                scored_actions.iter().enumerate().take(display_count)
            {
                let is_legal = legal.contains(action);
                let legal_str =
                    legal.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>().join(", ");
                assert!(
                    is_legal,
                    "Action {:?} is not legal, legal actions are {:?}",
                    action, legal_str
                );

                debug!(
                    "  Action {}: {:?} Score={:.4}, Visits={}, Reward={:.4}, Legal={}",
                    i + 1,
                    action,
                    score,
                    visits,
                    reward,
                    is_legal
                );
            }
        } else {
            debug!("No actions found to log");
        }
    }
}
