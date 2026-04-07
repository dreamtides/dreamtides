use std::cmp;
use std::cmp::Ordering;
use std::f64::consts;

use battle_mutations::actions::apply_battle_action;
use battle_mutations::player_mutations::player_state;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    ForPlayer, LegalActions, StandardLegalActions,
};
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::card_set::CardSet;
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

use crate::decision_log::{
    ActionResult, BudgetDetails, DecisionLogEntry, DepthLevelStats, TreeTraversalAccumulator,
};
use crate::position_assignment::{self, CharacterPlacement};
use crate::uct_config::UctConfig;
use crate::uct_tree::{SearchEdge, SearchGraph, SearchNode, SelectionMode};
use crate::{decision_log, log_search_results};

/// Monte Carlo search with heuristic-guided expansion for positioning nodes.
///
/// Based on V1's architecture (multi-step positioning in the tree, full
/// two-player UCT) with informed expansion: at positioning nodes, untried
/// actions are expanded in order of heuristic promise rather than arbitrarily.
pub fn search(
    initial_battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
) -> BattleAction {
    let legal = legal_actions::compute(initial_battle, player);
    let budget = compute_budget(&legal, config, initial_battle, player);

    let all_actions = legal.all();

    let action_results: Vec<_> = all_actions
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
    let num_actions = all_actions.len();
    let total_iterations = budget.iterations_per_action * num_actions as u32;
    let num_threads = rayon::current_num_threads();

    debug!(?total_iterations, ?action, ?num_threads, "Picked AI action (V4)");
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

    action
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
        let log_tree_stats = initial_battle.request_context.logging_options.log_ai_decisions;
        let mut traversal_stats =
            if log_tree_stats { Some(TreeTraversalAccumulator::default()) } else { None };

        for _ in 0..iterations_per_action {
            let mut battle = player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                randomize_player_rng.random(),
            );
            battle.request_context.logging_options.enable_action_legality_check = false;

            apply_battle_action::execute(&mut battle, player, action);

            let node =
                next_evaluation_target(&mut battle, &mut graph, root, traversal_stats.as_mut());
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
        let depth_stats = traversal_stats.map(TreeTraversalAccumulator::into_depth_stats);
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
            depth_stats,
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
    depth_stats: Option<Vec<DepthLevelStats>>,
}

/// Returns a descendant node to evaluate next for the provided parent node.
///
/// Same as V1's tree policy, but uses heuristic-guided expansion for
/// positioning nodes (`SelectPositioningCharacter` and `AssignColumn`).
/// Instead of expanding the first untried action arbitrarily, V4 picks
/// the most promising untried positioning action first based on position
/// assignment heuristics.
fn next_evaluation_target(
    battle: &mut BattleState,
    graph: &mut SearchGraph,
    from_node: NodeIndex,
    mut stats: Option<&mut TreeTraversalAccumulator>,
) -> NodeIndex {
    let mut node = from_node;
    let mut depth = 0usize;
    while let Some(player) = legal_actions::next_to_act(battle) {
        let actions = legal_actions::compute(battle, player);
        let explored = &graph[node].tried;

        let untried_action = match &actions {
            LegalActions::SelectPositioningCharacter { eligible } => {
                heuristic_expand_character(battle, player, eligible, explored)
            }
            LegalActions::AssignColumn { character, block_targets, attack_column } => {
                heuristic_expand_column(
                    battle,
                    player,
                    *character,
                    block_targets,
                    *attack_column,
                    explored,
                )
            }
            _ => actions.find_missing(explored),
        };

        if let Some(action) = untried_action {
            if let Some(s) = stats.as_mut() {
                s.record_expansion(depth, player, &action);
            }
            return add_child(battle, graph, player, node, action);
        } else {
            if let Some(s) = stats.as_mut() {
                s.record_selection(depth, player);
            }
            let best = best_child(graph, node, &actions, SelectionMode::Exploration);
            battle.request_context.logging_options.enable_action_legality_check = false;
            apply_battle_action::execute(battle, player, best.action);
            node = best.node;
        }
        depth += 1;
    }
    node
}

/// Heuristic expansion for character selection during positioning.
///
/// Generates position assignments and ranks untried characters by how
/// often they appear with `MoveToFrontRank` in those assignments.
/// Characters in more/better assignments are explored first. EndTurn
/// is explored first as an important baseline.
fn heuristic_expand_character(
    battle: &BattleState,
    player: PlayerName,
    eligible: &CardSet<CharacterId>,
    explored: &[BattleAction],
) -> Option<BattleAction> {
    let untried: Vec<BattleAction> = eligible
        .iter()
        .map(BattleAction::SelectCharacterForPositioning)
        .chain(std::iter::once(BattleAction::EndTurn))
        .filter(|a| !explored.contains(a))
        .collect();

    if untried.is_empty() {
        return None;
    }

    let assignments = position_assignment::generate(battle, player);
    if assignments.is_empty() {
        return untried.first().copied();
    }

    // Try EndTurn first if it hasn't been tried (important baseline).
    if untried.contains(&BattleAction::EndTurn) {
        return Some(BattleAction::EndTurn);
    }

    // For remaining characters, prefer those that appear in assignments
    // with MoveToFrontRank placements.
    let mut char_scores: Vec<(BattleAction, usize)> = untried
        .iter()
        .filter_map(|action| {
            if let BattleAction::SelectCharacterForPositioning(char_id) = action {
                let count = assignments
                    .iter()
                    .filter(|a| {
                        a.placements.iter().any(|(id, p)| {
                            *id == *char_id && matches!(p, CharacterPlacement::MoveToFrontRank(_))
                        })
                    })
                    .count();
                Some((*action, count))
            } else {
                None
            }
        })
        .collect();

    // Sort by score descending: characters in more assignments first.
    char_scores.sort_by(|a, b| b.1.cmp(&a.1));
    char_scores.first().map(|(action, _)| *action)
}

/// Heuristic expansion for column assignment during positioning.
///
/// Among untried columns, prefers blocking columns with highest opponent
/// spark (most points prevented), then attack columns.
fn heuristic_expand_column(
    battle: &BattleState,
    player: PlayerName,
    character: CharacterId,
    block_targets: &[u8],
    attack_column: Option<u8>,
    explored: &[BattleAction],
) -> Option<BattleAction> {
    let mut untried: Vec<BattleAction> = block_targets
        .iter()
        .map(|&col| BattleAction::MoveCharacterToFrontRank(character, col))
        .chain(attack_column.map(|col| BattleAction::MoveCharacterToFrontRank(character, col)))
        .filter(|a| !explored.contains(a))
        .collect();

    if untried.is_empty() {
        return None;
    }

    let opponent = player.opponent();

    // Sort by heuristic priority: block columns with highest opponent
    // spark first (most points prevented), then attack columns.
    untried.sort_by(|a, b| {
        let col_a = match a {
            BattleAction::MoveCharacterToFrontRank(_, c) => *c,
            _ => 0,
        };
        let col_b = match b {
            BattleAction::MoveCharacterToFrontRank(_, c) => *c,
            _ => 0,
        };

        let opp_spark_a = battle.cards.battlefield(opponent).front[col_a as usize]
            .and_then(|id| battle.cards.spark(opponent, id))
            .map_or(0, |s| s.0);
        let opp_spark_b = battle.cards.battlefield(opponent).front[col_b as usize]
            .and_then(|id| battle.cards.spark(opponent, id))
            .map_or(0, |s| s.0);

        // Higher opponent spark = higher priority (more points to prevent).
        opp_spark_b.cmp(&opp_spark_a)
    });

    untried.first().copied()
}

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

fn evaluate(battle: &mut BattleState, maximizing_player: PlayerName) -> OrderedFloat<f64> {
    while let Some(player) = legal_actions::next_to_act(battle) {
        let legal = legal_actions::compute(battle, player);
        let Some(action) = rollout_action(battle, player, &legal) else {
            panic_with!("No legal actions available", battle, player);
        };
        apply_battle_action::execute(battle, player, action);
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

fn rollout_action(
    battle: &BattleState,
    player: PlayerName,
    legal: &LegalActions,
) -> Option<BattleAction> {
    match legal {
        LegalActions::Standard { actions } => heuristic_standard_action(legal, actions),
        LegalActions::SelectPositioningCharacter { eligible } => {
            heuristic_select_positioning_character(battle, player, eligible)
        }
        LegalActions::AssignColumn { character, block_targets, attack_column } => {
            heuristic_assign_column(battle, player, *character, block_targets, *attack_column)
        }
        _ => legal.random_action(),
    }
}

fn heuristic_standard_action(
    legal: &LegalActions,
    actions: &StandardLegalActions,
) -> Option<BattleAction> {
    let action = legal.random_action()?;
    if action == BattleAction::EndTurn && actions.can_begin_positioning {
        Some(BattleAction::BeginPositioning)
    } else {
        Some(action)
    }
}

fn heuristic_select_positioning_character(
    battle: &BattleState,
    player: PlayerName,
    eligible: &CardSet<CharacterId>,
) -> Option<BattleAction> {
    if eligible.is_empty() {
        return Some(BattleAction::EndTurn);
    }

    let opponent = player.opponent();
    let opp_front = &battle.cards.battlefield(opponent).front;
    let has_block_targets = opp_front.iter().any(Option::is_some);

    if !has_block_targets {
        let opp_back_max_spark = battle
            .cards
            .battlefield(opponent)
            .back
            .iter()
            .filter_map(|slot| *slot)
            .filter_map(|id| battle.cards.spark(opponent, id))
            .map(|s| s.0)
            .max()
            .unwrap_or(0);

        let own_max_spark = eligible
            .iter()
            .filter_map(|id| battle.cards.spark(player, id))
            .map(|s| s.0)
            .max()
            .unwrap_or(0);

        if opp_back_max_spark > own_max_spark {
            return Some(BattleAction::EndTurn);
        }
    }

    let index = fastrand::usize(..eligible.len());
    eligible.iter().nth(index).map(BattleAction::SelectCharacterForPositioning)
}

fn heuristic_assign_column(
    battle: &BattleState,
    player: PlayerName,
    character: CharacterId,
    block_targets: &[u8],
    attack_column: Option<u8>,
) -> Option<BattleAction> {
    let own_spark = battle.cards.spark(player, character).unwrap_or_default().0;
    let opponent = player.opponent();

    let mut best_block_col = None;
    let mut best_block_spark = 0u32;
    for &col in block_targets {
        if let Some(opp_id) = battle.cards.battlefield(opponent).front[col as usize] {
            let opp_spark = battle.cards.spark(opponent, opp_id).unwrap_or_default().0;
            if opp_spark > best_block_spark {
                best_block_spark = opp_spark;
                best_block_col = Some(col);
            }
        }
    }

    if let Some(col) = best_block_col
        && best_block_spark >= own_spark
    {
        return Some(BattleAction::MoveCharacterToFrontRank(character, col));
    }

    if let Some(col) = attack_column {
        return Some(BattleAction::MoveCharacterToFrontRank(character, col));
    }

    if !block_targets.is_empty() {
        let index = fastrand::usize(..block_targets.len());
        Some(BattleAction::MoveCharacterToFrontRank(character, block_targets[index]))
    } else {
        None
    }
}

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

    let chosen_action = best_result.action;
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
                depth_stats: if r.action == chosen_action { r.depth_stats.clone() } else { None },
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
