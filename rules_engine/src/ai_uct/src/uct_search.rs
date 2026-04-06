use std::cmp;
use std::cmp::Ordering;
use std::f64::consts;

use battle_mutations::actions::apply_battle_action;
use battle_mutations::player_mutations::player_state;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{ForPlayer, LegalActions};
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use chrono::Utc;
use core_data::types::PlayerName;
use ordered_float::OrderedFloat;
use petgraph::Direction;
use petgraph::prelude::NodeIndex;
use petgraph::visit::EdgeRef;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use rayon::prelude::*;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

use crate::decision_log::{ActionResult, BudgetDetails, DecisionLogEntry};
use crate::uct_config::UctConfig;
use crate::uct_tree::{SearchEdge, SearchGraph, SearchNode, SelectionMode};
use crate::{decision_log, log_search_results};

/// Monte Carlo search algorithm.
///
/// Searches for an action for `player` to take in the given `battle` state. The
/// provided `graph` and `root` should correspond to a search graph rooted at
/// this state, i.e. one where the agent's possible actions form outgoing edges.
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
    initial_battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
) -> BattleAction {
    let legal = legal_actions::compute(initial_battle, player);
    let budget = compute_budget(&legal, config, initial_battle, player);

    let action_results: Vec<_> = legal
        .all()
        .par_iter()
        .with_min_len(if config.single_threaded { usize::MAX } else { 1 })
        .map(|&action| {
            search_action_candidate(
                initial_battle,
                player,
                budget.iterations_per_action,
                action,
                None,
            )
        })
        .collect();

    let Some(best_result) = action_results.iter().max_by_key(|result| {
        if result.visit_count == 0 {
            OrderedFloat(-f64::INFINITY)
        } else {
            OrderedFloat(result.total_reward.0 / result.visit_count as f64)
        }
    }) else {
        panic_with!("No legal actions available", initial_battle, player);
    };

    let action = best_result.action;
    let num_actions = legal.len();
    let total_iterations = budget.iterations_per_action * num_actions as u32;
    let num_threads = rayon::current_num_threads();

    debug!(?total_iterations, ?action, ?num_threads, "Picked AI action");
    if initial_battle.request_context.logging_options.log_ai_search_diagram {
        log_search_results::log_results_diagram(
            &best_result.graph,
            best_result.root,
            action,
            &initial_battle.request_context,
        );
    }

    if initial_battle.request_context.logging_options.log_ai_decisions {
        write_decision_log_entry(
            initial_battle,
            player,
            &action_results,
            best_result,
            &budget,
            num_actions,
            num_threads,
        );
    }

    // I've experimented with persisting the search tree to reuse in future
    // searches, and it does improve win rates somewhat, but the
    // complexity/performance costs seem to not be worth it.

    action
}

/// Public version of `search_action_candidate` for use in benchmark tests.
pub fn search_first_action_candidate_for_benchmarking(
    initial_battle: &BattleState,
    player: PlayerName,
) -> BattleAction {
    fastrand::seed(31415926535897);
    let legal = legal_actions::compute(initial_battle, player);
    let all_actions = legal.all();
    let action = all_actions.first().expect("No legal actions available");
    let result = search_action_candidate(initial_battle, player, 10, *action, Some(31415926535897));
    result.action
}

/// Public version of `evaluate` for use in benchmark tests.
pub fn evaluate_for_benchmarking(
    battle: &mut BattleState,
    maximizing_player: PlayerName,
) -> OrderedFloat<f64> {
    fastrand::seed(31415926535897);
    evaluate(battle, maximizing_player)
}

fn search_action_candidate(
    initial_battle: &BattleState,
    player: PlayerName,
    iterations_per_action: u32,
    action: BattleAction,
    randomize_player_seed: Option<u64>,
) -> ActionSearchResult {
    let subscriber = tracing_subscriber::registry().with(EnvFilter::new("warn"));
    tracing::subscriber::with_default(subscriber, || {
        let mut graph = SearchGraph::default();
        let root = graph.add_node(SearchNode {
            player,
            total_reward: OrderedFloat(0.0),
            visit_count: 0,
            tried: Vec::new(),
        });

        let mut randomize_player_rng = Xoshiro256PlusPlus::seed_from_u64(
            randomize_player_seed.unwrap_or_else(|| rand::rng().random()),
        );

        let mut wins = 0u32;
        let mut losses = 0u32;
        let mut draws = 0u32;

        for _ in 0..iterations_per_action {
            // Use a different random state every time. Doing this less
            // frequently does improve performance, but also pretty
            // consistently reduces play skill.
            let mut battle = player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                randomize_player_rng.random(),
            );
            battle.request_context.logging_options.enable_action_legality_check = false;

            apply_battle_action::execute(&mut battle, player, action);

            let node = next_evaluation_target(&mut battle, &mut graph, root);
            let reward = evaluate(&mut battle, player);
            back_propagate_rewards(&mut graph, player, node, reward);

            match reward.0 {
                r if r > 0.0 => wins += 1,
                r if r < 0.0 => losses += 1,
                _ => draws += 1,
            }
        }

        let total_reward = graph[root].total_reward;
        let visit_count = graph[root].visit_count;
        let tree_node_count = graph.node_count();
        let tree_max_depth = crate::decision_log::compute_tree_depth(&graph, root);
        ActionSearchResult {
            action,
            graph,
            root,
            total_reward,
            visit_count,
            wins,
            losses,
            draws,
            tree_node_count,
            tree_max_depth,
        }
    })
}

struct ActionSearchResult {
    action: BattleAction,
    graph: SearchGraph,
    root: NodeIndex,
    total_reward: OrderedFloat<f64>,
    visit_count: u32,
    wins: u32,
    losses: u32,
    draws: u32,
    tree_node_count: usize,
    tree_max_depth: u32,
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
    let mut node = from_node;
    while let Some(player) = legal_actions::next_to_act(battle) {
        let actions = legal_actions::compute(battle, player);
        let explored = &graph[node].tried;
        // Keeping track of tried actions on the node is a small performance boost
        // over iterating through edges (~3% benchmark improvement).
        if let Some(action) = actions.find_missing(explored) {
            // An action exists from this node which has not yet been tried
            return add_child(battle, graph, player, node, action);
        } else {
            // All actions from this node have been tried, recursively search
            // the best candidate
            let best = best_child(graph, node, &actions, SelectionMode::Exploration);
            battle.request_context.logging_options.enable_action_legality_check = false;
            apply_battle_action::execute(battle, player, best.action);
            node = best.node;
        }
    }
    node
}

/// Adds a new child node to the search graph (the 'expand' function).
///
/// Generates a new tree node by applying the next untried action from the
/// provided input node. Mutates the provided [BattleState] to apply the
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
fn add_child(
    battle: &mut BattleState,
    graph: &mut SearchGraph,
    player: PlayerName,
    parent: NodeIndex,
    action: BattleAction,
) -> NodeIndex {
    battle.request_context.logging_options.enable_action_legality_check = false;
    graph[parent].tried.push(action);
    apply_battle_action::execute(battle, player, action);
    let child = graph.add_node(SearchNode {
        player,
        total_reward: OrderedFloat(0.0),
        visit_count: 0,
        tried: Vec::new(),
    });
    graph.add_edge(parent, child, SearchEdge { action });
    child
}

struct BestChild {
    action: BattleAction,
    node: NodeIndex,
}

/// Picks the most promising child node to explore next.
///
/// This picks a child based on its computed reward and visit count, subject to
/// the requested [SelectionMode].
///
/// Returns the action taken to produce this child and its associated node.
///
/// Panics if the provided `from_node` has no legal children.
fn best_child(
    graph: &SearchGraph,
    from_node: NodeIndex,
    legal: &LegalActions,
    selection_mode: SelectionMode,
) -> BestChild {
    let parent_visits = graph[from_node].visit_count;

    graph
        .edges(from_node)
        .filter(|e| legal.contains(e.weight().action, ForPlayer::Agent))
        .max_by_key(|edge| {
            let target = edge.target();
            child_score(
                parent_visits,
                graph[target].visit_count,
                graph[target].total_reward,
                selection_mode,
            )
        })
        .map(|edge| BestChild { action: edge.weight().action, node: edge.target() })
        .expect("No legal children available")
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
    reward: OrderedFloat<f64>,
) {
    let mut node = leaf_node;
    loop {
        let weight = &mut graph[node];
        weight.visit_count += 1;
        weight.total_reward += if weight.player == maximizing_player { reward } else { -reward };

        node = match graph.neighbors_directed(node, Direction::Incoming).next() {
            Some(n) => n,
            _ => break,
        };
    }
}

/// Scores a given [BattleState] for the maximizing player (the 'default policy'
/// of the search).
///
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
fn evaluate(battle: &mut BattleState, maximizing_player: PlayerName) -> OrderedFloat<f64> {
    while let Some(player) = legal_actions::next_to_act(battle) {
        let Some(action) = legal_actions::compute(battle, player).random_action() else {
            panic_with!("No legal actions available", battle, player);
        };
        apply_battle_action::execute(battle, player, action);

        // I've tried aborting early here and using heuristics to evaluate the
        // battle state, but this has substantially worse win rates.
    }

    let BattleStatus::GameOver { winner } = battle.status else {
        panic_with!("Battle has not ended", battle);
    };
    let reward = if winner == Some(maximizing_player) {
        1.0
    } else if winner.is_some() {
        -1.0
    } else {
        0.0
    };
    OrderedFloat(reward)
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
    reward: OrderedFloat<f64>,
    selection_mode: SelectionMode,
) -> OrderedFloat<f64> {
    let exploitation = reward / f64::from(child_visits);
    let exploration =
        f64::sqrt((2.0 * f64::ln(f64::from(parent_visits))) / f64::from(child_visits));
    let exploration_bias = match selection_mode {
        SelectionMode::Exploration => consts::FRAC_1_SQRT_2,
        SelectionMode::RewardOnly => 0.0,
    };
    exploitation + (exploration_bias * exploration)
}

struct BudgetInfo {
    iterations_per_action: u32,
    base_iterations: u32,
    multiplier: f64,
    multiplier_reason: &'static str,
}

/// Calculates the number of iterations to run per action based on the legal
/// actions available and configuration parameters.
///
/// The calculation prioritizes distributing iterations evenly across available
/// actions while respecting configured limits. Prompt actions receive fewer
/// iterations as they require faster response times.
fn compute_budget(
    legal: &LegalActions,
    config: &UctConfig,
    battle: &BattleState,
    agent: PlayerName,
) -> BudgetInfo {
    let base_iterations = match legal.len() {
        0 => config.max_iterations_per_action,
        action_count => {
            let total_budget =
                config.max_iterations_per_action * config.max_total_actions_multiplier;
            let distributed_iterations = (total_budget as f64 / action_count as f64) as u32;
            cmp::min(distributed_iterations, config.max_iterations_per_action)
        }
    };

    let is_main =
        battle.turn.active_player == agent && matches!(battle.phase, BattleTurnPhase::Main);
    let player_state = battle.players.player(battle.turn.active_player);

    let (multiplier, multiplier_reason) = match is_main {
        _ if legal.is_prompt() => (0.5, "prompt"),
        true if player_state.current_energy >= player_state.produced_energy => (1.5, "first_main"),
        true => (1.0, "main"),
        _ => (0.75, "other"),
    };
    let (applied_multiplier, applied_reason) = match config.iteration_multiplier_override {
        Some(m) => (m, "override"),
        None => (multiplier, multiplier_reason),
    };
    BudgetInfo {
        iterations_per_action: ((base_iterations as f64) * applied_multiplier) as u32,
        base_iterations,
        multiplier: applied_multiplier,
        multiplier_reason: applied_reason,
    }
}

fn write_decision_log_entry(
    battle: &BattleState,
    player: PlayerName,
    action_results: &[ActionSearchResult],
    best_result: &ActionSearchResult,
    budget: &BudgetInfo,
    num_actions: usize,
    num_threads: usize,
) {
    let best_avg = if best_result.visit_count == 0 {
        0.0
    } else {
        best_result.total_reward.0 / best_result.visit_count as f64
    };

    let mut results: Vec<ActionResult> = action_results
        .iter()
        .map(|r| {
            let avg =
                if r.visit_count == 0 { 0.0 } else { r.total_reward.0 / r.visit_count as f64 };
            ActionResult {
                action: format!("{:?}", r.action),
                action_short: r.action.battle_action_string(),
                total_reward: r.total_reward.0,
                visit_count: r.visit_count,
                avg_reward: avg,
                wins: r.wins,
                losses: r.losses,
                draws: r.draws,
                tree_node_count: r.tree_node_count,
                tree_max_depth: r.tree_max_depth,
            }
        })
        .collect();
    results.sort_by(|a, b| b.avg_reward.partial_cmp(&a.avg_reward).unwrap_or(Ordering::Equal));

    let entry = DecisionLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        player: format!("{:?}", player),
        chosen_action: format!("{:?}", best_result.action),
        chosen_action_short: best_result.action.battle_action_string(),
        chosen_avg_reward: best_avg,
        game_state: decision_log::build_game_state_snapshot(battle),
        budget: BudgetDetails {
            iterations_per_action: budget.iterations_per_action,
            base_iterations: budget.base_iterations,
            total_iterations: budget.iterations_per_action * num_actions as u32,
            num_actions,
            multiplier: budget.multiplier,
            multiplier_reason: budget.multiplier_reason.to_string(),
            num_threads,
        },
        action_results: results,
    };
    decision_log::write_decision_log(&entry, &battle.request_context);
}
