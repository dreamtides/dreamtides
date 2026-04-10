use std::cmp;
use std::cmp::Ordering;
use std::f64::consts;

use battle_mutations::actions::apply_battle_action;
use battle_mutations::player_mutations::player_state;
use battle_queries::battle_card_queries::card_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    ForPlayer, LegalActions, PrimaryLegalAction, StandardLegalActions,
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
    self, ActionResult, BudgetDetails, DecisionLogEntry, DepthLevelStats, TreeTraversalAccumulator,
};
use crate::position_assignment::{self, PositionAssignment};
use crate::rollout_policy;
use crate::uct_config::UctConfig;
use crate::uct_tree::{SearchEdge, SearchGraph, SearchNode, SelectionMode};

/// Result returned by the V3 search, including the immediate action and any
/// pending position assignment to be executed as follow-up actions.
pub struct V3SearchResult {
    pub action: BattleAction,
    pub assignment: Option<PositionAssignment>,
}

/// Monte Carlo search with heuristic position assignments and two-player UCT.
///
/// Combines V2's atomic position assignments as root-level candidates with
/// V1's full two-player UCT tree policy and randomized rollouts.
pub fn search(
    initial_battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
) -> V3SearchResult {
    let legal = legal_actions::compute(initial_battle, player);
    let candidates = build_candidates(initial_battle, player, &legal);
    let budget = compute_budget(&candidates, config, initial_battle, player);

    let candidate_results: Vec<_> = candidates
        .into_par_iter()
        .with_min_len(if config.single_threaded { usize::MAX } else { 1 })
        .map(|candidate| {
            let iterations = budget.iterations_for(&candidate);
            search_candidate(initial_battle, player, iterations, candidate, None)
        })
        .collect();

    let Some((best_index, best_result)) =
        candidate_results.iter().enumerate().max_by_key(|(_, result)| {
            if result.visit_count == 0 {
                OrderedFloat(-f64::INFINITY)
            } else {
                OrderedFloat(result.total_reward.0 / result.visit_count as f64)
            }
        })
    else {
        panic_with!("No legal actions available", initial_battle, player);
    };

    let result = match &best_result.candidate {
        SearchCandidate::Action(a) => V3SearchResult { action: *a, assignment: None },
        SearchCandidate::Assignment(a) => {
            V3SearchResult { action: BattleAction::BeginPositioning, assignment: Some(a.clone()) }
        }
    };

    let num_actions = candidate_results.len();
    let total_iterations: u32 = candidate_results.iter().map(|r| r.visit_count).sum();
    let num_threads = rayon::current_num_threads();

    debug!(?total_iterations, action = ?result.action, ?num_threads, "Picked AI action (V3)");

    if initial_battle.request_context.logging_options.log_ai_decisions {
        write_decision_log_entry(
            initial_battle,
            player,
            &candidate_results,
            best_index,
            &budget,
            num_actions,
            num_threads,
        );
    }

    result
}

#[derive(Debug, Clone)]
enum SearchCandidate {
    Action(BattleAction),
    Assignment(PositionAssignment),
}

struct CandidateSearchResult {
    candidate: SearchCandidate,
    total_reward: OrderedFloat<f64>,
    visit_count: u32,
    wins: u32,
    losses: u32,
    draws: u32,
    tree_node_count: usize,
    tree_max_depth: u32,
    depth_stats: Option<Vec<DepthLevelStats>>,
}

struct BudgetInfo {
    iterations_per_action: u32,
    iterations_per_assignment: u32,
    base_iterations: u32,
    multiplier: f64,
    multiplier_reason: &'static str,
}

impl BudgetInfo {
    fn iterations_for(&self, candidate: &SearchCandidate) -> u32 {
        match candidate {
            SearchCandidate::Action(_) => self.iterations_per_action,
            SearchCandidate::Assignment(_) => self.iterations_per_assignment,
        }
    }
}

fn build_candidates(
    battle: &BattleState,
    player: PlayerName,
    legal: &LegalActions,
) -> Vec<SearchCandidate> {
    let mut candidates = Vec::new();
    let LegalActions::Standard { actions } = legal else {
        if let Some(action) = legal.random_action() {
            return vec![SearchCandidate::Action(action)];
        }
        return Vec::new();
    };

    let primary_action = match actions.primary {
        PrimaryLegalAction::EndTurn => BattleAction::EndTurn,
        PrimaryLegalAction::PassPriority => BattleAction::PassPriority,
        PrimaryLegalAction::StartNextTurn => BattleAction::StartNextTurn,
    };
    candidates.push(SearchCandidate::Action(primary_action));

    for card_id in actions.play_card_from_hand.iter() {
        candidates.push(SearchCandidate::Action(BattleAction::PlayCardFromHand(card_id)));
    }
    for card_id in actions.play_card_from_void.iter() {
        candidates.push(SearchCandidate::Action(BattleAction::PlayCardFromVoid(card_id)));
    }
    for char_id in actions.activate_abilities_for_character.iter() {
        candidates
            .push(SearchCandidate::Action(BattleAction::ActivateAbilityForCharacter(char_id)));
    }
    for &(char_id, pos) in &actions.reposition_to_front {
        candidates
            .push(SearchCandidate::Action(BattleAction::MoveCharacterToFrontRank(char_id, pos)));
    }
    for &(char_id, pos) in &actions.reposition_to_back {
        candidates
            .push(SearchCandidate::Action(BattleAction::MoveCharacterToBackRank(char_id, pos)));
    }

    if actions.can_begin_positioning {
        let assignments = position_assignment::generate(battle, player);
        for assignment in assignments {
            candidates.push(SearchCandidate::Assignment(assignment));
        }
    }

    candidates
}

fn search_candidate(
    initial_battle: &BattleState,
    player: PlayerName,
    iterations: u32,
    candidate: SearchCandidate,
    randomize_player_seed: Option<u64>,
) -> CandidateSearchResult {
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

        for _ in 0..iterations {
            let mut battle = player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                randomize_player_rng.random(),
            );
            battle.request_context.logging_options.enable_action_legality_check = false;

            apply_candidate(&mut battle, player, &candidate);

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
        CandidateSearchResult {
            candidate,
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

fn apply_candidate(battle: &mut BattleState, player: PlayerName, candidate: &SearchCandidate) {
    match candidate {
        SearchCandidate::Action(action) => {
            apply_battle_action::execute(battle, player, *action);
        }
        SearchCandidate::Assignment(assignment) => {
            rollout_policy::apply_position_assignment(battle, player, assignment);
        }
    }
}

/// Two-player UCT tree policy (from V1). Expands tree nodes for ALL players
/// using UCB1 selection, not just the maximizing player.
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
        if let Some(action) = actions.find_missing(explored) {
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

/// Rollout using randomized logic with atomic position assignments.
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

/// Selects an action during rollout with heuristic overrides for positioning.
///
/// Uses V1's domain-aware heuristics: biases toward positioning when
/// available, makes smart blocking/attacking decisions based on spark
/// values, but retains randomization for character selection.
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

/// Always positions when there are opponent front-rank characters to block.
/// When there are no block targets, checks whether the opponent has
/// threatening back-rank characters. If the opponent's largest back-rank
/// threat exceeds the player's largest eligible character, saves characters
/// for future blocking instead.
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
            .filter_map(|id| card_properties::spark(battle, opponent, id))
            .map(|s| s.0)
            .max()
            .unwrap_or(0);

        let own_max_spark = eligible
            .iter()
            .filter_map(|id| card_properties::spark(battle, player, id))
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

/// Blocks the highest-spark opponent character when their spark exceeds
/// this character's spark. Otherwise attacks in an empty column if
/// available.
fn heuristic_assign_column(
    battle: &BattleState,
    player: PlayerName,
    character: CharacterId,
    block_targets: &[u8],
    attack_column: Option<u8>,
) -> Option<BattleAction> {
    let own_spark = card_properties::spark(battle, player, character).unwrap_or_default().0;
    let opponent = player.opponent();

    let mut best_block_col = None;
    let mut best_block_spark = 0u32;
    for &col in block_targets {
        if let Some(opp_id) = battle.cards.battlefield(opponent).front[col as usize] {
            let opp_spark = card_properties::spark(battle, opponent, opp_id).unwrap_or_default().0;
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
    graph.add_edge(parent, child, SearchEdge { action, prior: 0.0 });
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

/// Position candidates collectively count as 2 action "slots" for budget
/// distribution, keeping action candidate quality high while still
/// evaluating diverse positioning options.
fn compute_budget(
    candidates: &[SearchCandidate],
    config: &UctConfig,
    battle: &BattleState,
    agent: PlayerName,
) -> BudgetInfo {
    let action_count =
        candidates.iter().filter(|c| matches!(c, SearchCandidate::Action(_))).count();
    let assignment_count =
        candidates.iter().filter(|c| matches!(c, SearchCandidate::Assignment(_))).count();

    let total_budget = config.max_iterations_per_action * config.max_total_actions_multiplier;

    // Position assignments collectively count as 2 action slots.
    let effective_slots = action_count as f64 + if assignment_count > 0 { 2.0 } else { 0.0 };

    let (iterations_per_action, iterations_per_assignment) = if effective_slots == 0.0 {
        (config.max_iterations_per_action, config.max_iterations_per_action)
    } else {
        let per_slot = total_budget as f64 / effective_slots;
        let per_action = cmp::min(per_slot as u32, config.max_iterations_per_action);
        let per_assignment = if assignment_count > 0 {
            cmp::min(
                (2.0 * per_slot / assignment_count as f64) as u32,
                config.max_iterations_per_action,
            )
        } else {
            0
        };
        (per_action, per_assignment)
    };

    let is_main =
        battle.turn.active_player == agent && matches!(battle.phase, BattleTurnPhase::Main);
    let player_state = battle.players.player(battle.turn.active_player);

    let is_prompt = !matches!(legal_actions::compute(battle, agent), LegalActions::Standard { .. });

    let (multiplier, multiplier_reason) = if is_prompt {
        (0.5, "prompt")
    } else if is_main && player_state.current_energy >= player_state.produced_energy {
        (1.5, "first_main")
    } else if is_main {
        (1.0, "main")
    } else {
        (0.75, "other")
    };

    let (applied_multiplier, applied_reason) = match config.iteration_multiplier_override {
        Some(m) => (m, "override"),
        None => (multiplier, multiplier_reason),
    };

    BudgetInfo {
        iterations_per_action: ((iterations_per_action as f64) * applied_multiplier) as u32,
        iterations_per_assignment: ((iterations_per_assignment as f64) * applied_multiplier) as u32,
        base_iterations: (total_budget as f64 / effective_slots) as u32,
        multiplier: applied_multiplier,
        multiplier_reason: applied_reason,
    }
}

fn candidate_description(
    battle: &BattleState,
    player: PlayerName,
    candidate: &SearchCandidate,
) -> String {
    match candidate {
        SearchCandidate::Action(action) => format!("{:?}", action),
        SearchCandidate::Assignment(assignment) => {
            position_assignment::describe(battle, player, assignment)
        }
    }
}

fn candidate_short_description(
    battle: &BattleState,
    player: PlayerName,
    candidate: &SearchCandidate,
) -> String {
    match candidate {
        SearchCandidate::Action(action) => action.battle_action_string(),
        SearchCandidate::Assignment(assignment) => {
            position_assignment::describe(battle, player, assignment)
        }
    }
}

fn write_decision_log_entry(
    battle: &BattleState,
    player: PlayerName,
    candidate_results: &[CandidateSearchResult],
    best_index: usize,
    budget: &BudgetInfo,
    num_actions: usize,
    num_threads: usize,
) {
    let best_result = &candidate_results[best_index];
    let best_avg = if best_result.visit_count == 0 {
        0.0
    } else {
        best_result.total_reward.0 / best_result.visit_count as f64
    };

    let total_iterations: u32 = candidate_results.iter().map(|r| r.visit_count).sum();

    let mut results: Vec<ActionResult> = candidate_results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let avg =
                if r.visit_count == 0 { 0.0 } else { r.total_reward.0 / r.visit_count as f64 };
            ActionResult {
                action: candidate_description(battle, player, &r.candidate),
                action_short: candidate_short_description(battle, player, &r.candidate),
                total_reward: r.total_reward.0,
                visit_count: r.visit_count,
                avg_reward: avg,
                wins: r.wins,
                losses: r.losses,
                draws: r.draws,
                tree_node_count: r.tree_node_count,
                tree_max_depth: r.tree_max_depth,
                depth_stats: if i == best_index { r.depth_stats.clone() } else { None },
            }
        })
        .collect();
    results.sort_by(|a, b| b.avg_reward.partial_cmp(&a.avg_reward).unwrap_or(Ordering::Equal));

    let entry = DecisionLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        player: format!("{:?}", player),
        chosen_action: candidate_description(battle, player, &best_result.candidate),
        chosen_action_short: candidate_short_description(battle, player, &best_result.candidate),
        chosen_avg_reward: best_avg,
        game_state: decision_log::build_game_state_snapshot(battle),
        budget: BudgetDetails {
            iterations_per_action: budget.iterations_per_action,
            base_iterations: budget.base_iterations,
            total_iterations,
            num_actions,
            multiplier: budget.multiplier,
            multiplier_reason: budget.multiplier_reason.to_string(),
            num_threads,
        },
        action_results: results,
    };
    decision_log::write_decision_log(&entry, &battle.request_context);
}
