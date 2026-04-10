use std::cmp::Ordering;
use std::f64::consts;
use std::time::{Duration, Instant};

use battle_mutations::actions::apply_battle_action;
use battle_mutations::player_mutations::player_state;
use battle_queries::battle_card_queries::card_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    ForPlayer, LegalActions, StandardLegalActions,
};
use battle_queries::panic_with;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
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

use crate::decision_log::TreeTraversalAccumulator;
use crate::hybrid_decision_log::{self, DecisionLogEntryHybrid, RootActionLog};
use crate::position_assignment::{self, CharacterPlacement};
use crate::rollout_policy::{self, HybridRolloutChoice};
use crate::uct_tree::{SearchEdge, SearchGraph, SearchNode, SelectionMode};
use crate::{decision_log, hybrid_eval, hybrid_features, hybrid_model};

const MIN_ITERATIONS_PER_ACTION: u32 = 64;
const ITERATION_BATCH_SIZE: u32 = 16;
const MAX_ROLLOUT_ACTIONS: u32 = 24;
const MAX_ROLLOUT_HANDOFFS: u32 = 2;
const MAX_RESOLUTION_ACTIONS: u32 = 12;
const MAX_PARALLEL_ROOT_BATCHES: usize = 4;
const HEURISTIC_PRIOR_WEIGHT: f64 = 0.35;
const PRIOR_EPSILON: f64 = 1e-6;
const PRIOR_TEMPERATURE: f64 = 1.5;
const ROOT_PRIOR_WEIGHT: f64 = 0.7;
const ROOT_UNIFORM_WEIGHT: f64 = 0.3;
const DEADLINE_SAFETY_MS: u64 = 25;

pub fn is_available() -> bool {
    hybrid_model::load_model().is_some()
}

pub fn search(initial_battle: &BattleState, player: PlayerName, budget_ms: u32) -> BattleAction {
    let start = Instant::now();
    let Some(model) = hybrid_model::load_model() else {
        panic_with!("MonteCarloHybridV1 requires trained policy artifacts", initial_battle, player);
    };
    let deadline = Instant::now()
        + Duration::from_millis(u64::from(budget_ms).saturating_sub(DEADLINE_SAFETY_MS));
    let legal = legal_actions::compute(initial_battle, player);
    let state_features = hybrid_features::extract_state_features(initial_battle, player);
    let legal_action_count = legal.len();
    let all_actions = legal.all();
    let mut states: Vec<_> = all_actions
        .iter()
        .map(|&action| {
            let heuristic_prior_score =
                heuristic_prior_score(initial_battle, player, &legal, action);
            RootActionState::new(
                action,
                initial_battle,
                player,
                &state_features,
                legal_action_count,
                model.score(&hybrid_features::merge_policy_features(
                    &state_features,
                    &hybrid_features::extract_action_features(initial_battle, player, action),
                    legal_action_count,
                )) + HEURISTIC_PRIOR_WEIGHT * (heuristic_prior_score + 1.0),
                heuristic_prior_score,
            )
        })
        .collect();
    let flat_prior = assign_prior_shares(&mut states);

    while Instant::now() < deadline {
        let schedule = build_schedule(&states);
        if schedule.iter().all(|iterations| *iterations == 0) {
            break;
        }
        states.par_iter_mut().enumerate().for_each(|(index, state)| {
            let iterations = schedule[index];
            if iterations > 0 {
                run_iterations(state, initial_battle, player, iterations, deadline);
            }
        });
    }

    let chosen = states
        .iter()
        .max_by(|left, right| state_avg_reward(left).total_cmp(&state_avg_reward(right)))
        .unwrap_or_else(|| panic_with!("No legal actions available", initial_battle, player));
    write_decision_log(
        initial_battle,
        player,
        budget_ms,
        legal_action_count,
        &states,
        flat_prior,
        chosen,
        start.elapsed(),
    );
    chosen.action
}

struct BestChild {
    action: BattleAction,
    node: NodeIndex,
}

struct RootActionState {
    action: BattleAction,
    cutoff_count: u32,
    draws: u32,
    graph: SearchGraph,
    losses: u32,
    pass_suppression_count: u32,
    heuristic_prior_score: f64,
    prior_score: f64,
    prior_share: f64,
    randomize_player_rng: Xoshiro256PlusPlus,
    root: NodeIndex,
    rollout_action_count: u32,
    wins: u32,
}

impl RootActionState {
    fn new(
        action: BattleAction,
        initial_battle: &BattleState,
        player: PlayerName,
        state_features: &crate::hybrid_dataset::FeatureMap,
        legal_action_count: usize,
        prior_score: f64,
        heuristic_prior_score: f64,
    ) -> Self {
        let mut graph = SearchGraph::default();
        let root = graph.add_node(SearchNode {
            player,
            total_reward: OrderedFloat(0.0),
            visit_count: 0,
            tried: Vec::new(),
        });
        let _ = (initial_battle, state_features, legal_action_count);
        Self {
            action,
            cutoff_count: 0,
            draws: 0,
            graph,
            heuristic_prior_score,
            losses: 0,
            pass_suppression_count: 0,
            prior_score,
            prior_share: 0.0,
            randomize_player_rng: Xoshiro256PlusPlus::seed_from_u64(rand::rng().random()),
            root,
            rollout_action_count: 0,
            wins: 0,
        }
    }
}

fn assign_prior_shares(states: &mut [RootActionState]) -> bool {
    if states.is_empty() {
        return true;
    }

    let clamped_scores: Vec<f64> = states
        .iter()
        .map(|state| if state.prior_score.is_finite() { state.prior_score.max(0.0) } else { 0.0 })
        .collect();
    let min_score = clamped_scores.iter().copied().fold(f64::INFINITY, f64::min);
    let max_score = clamped_scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let flat_prior = (max_score - min_score).abs() < PRIOR_EPSILON
        || clamped_scores.iter().copied().sum::<f64>() <= PRIOR_EPSILON;

    if flat_prior {
        let uniform = 1.0 / states.len() as f64;
        for state in states {
            state.prior_share = uniform;
        }
        return true;
    }

    let max_clamped = clamped_scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let exp_scores: Vec<f64> = clamped_scores
        .iter()
        .map(|score| ((score - max_clamped) / PRIOR_TEMPERATURE).exp())
        .collect();
    let total = exp_scores.iter().copied().sum::<f64>().max(PRIOR_EPSILON);
    for (state, exp_score) in states.iter_mut().zip(exp_scores) {
        state.prior_share = exp_score / total;
    }
    false
}

fn heuristic_prior_score(
    battle: &BattleState,
    player: PlayerName,
    legal: &LegalActions,
    action: BattleAction,
) -> f64 {
    match legal {
        LegalActions::Standard { actions } => {
            (rollout_policy::score_standard_action(battle, player, actions, action) / 100.0)
                .clamp(-1.0, 1.0)
        }
        LegalActions::SelectPositioningCharacter { eligible: _ } => match action {
            BattleAction::EndTurn => -0.6,
            BattleAction::SelectCharacterForPositioning(character_id) => {
                let assignments = position_assignment::generate(battle, player);
                if assignments.is_empty() {
                    0.0
                } else {
                    let usage = assignments
                        .iter()
                        .filter(|assignment| {
                            assignment.placements.iter().any(|(id, placement)| {
                                *id == character_id
                                    && matches!(placement, CharacterPlacement::MoveToFrontRank(_))
                            })
                        })
                        .count() as f64;
                    (usage / assignments.len() as f64).clamp(0.0, 1.0)
                }
            }
            _ => 0.0,
        },
        LegalActions::AssignColumn { character, block_targets, attack_column } => match action {
            BattleAction::MoveCharacterToFrontRank(_, column) => {
                let own_spark =
                    card_properties::spark(battle, player, *character).unwrap_or_default().0;
                let opponent = player.opponent();
                let enemy_spark = battle.cards.battlefield(opponent).front[column as usize]
                    .and_then(|id| card_properties::spark(battle, opponent, id))
                    .map_or(0, |spark| spark.0);
                let is_attack = attack_column.is_some_and(|attack| attack == column);
                let is_block = block_targets.contains(&column);
                let trade_bonus = if enemy_spark >= own_spark { 0.3 } else { 0.1 };
                let role_bonus = if is_block {
                    0.2
                } else if is_attack {
                    0.1
                } else {
                    0.0
                };
                ((f64::from(enemy_spark.min(8)) / 8.0) + trade_bonus + role_bonus).clamp(0.0, 1.0)
            }
            _ => 0.0,
        },
        _ => 0.0,
    }
}

fn build_schedule(states: &[RootActionState]) -> Vec<u32> {
    let selected =
        states.len().min(rayon::current_num_threads().max(1)).min(MAX_PARALLEL_ROOT_BATCHES);
    let total_iterations: u32 = states.iter().map(state_visit_count).sum();
    let uniform_share = 1.0 / states.len().max(1) as f64;

    let mut priorities: Vec<_> = states
        .iter()
        .enumerate()
        .map(|(index, state)| {
            let visits = state_visit_count(state);
            let priority = if visits < MIN_ITERATIONS_PER_ACTION {
                f64::INFINITY - visits as f64
            } else {
                let target_share =
                    ROOT_PRIOR_WEIGHT * state.prior_share + ROOT_UNIFORM_WEIGHT * uniform_share;
                let current_share = if total_iterations == 0 {
                    0.0
                } else {
                    f64::from(visits) / f64::from(total_iterations)
                };
                target_share - current_share
            };
            (index, priority)
        })
        .collect();
    priorities.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(Ordering::Equal));

    let mut schedule = vec![0; states.len()];
    for (index, _) in priorities.into_iter().take(selected) {
        schedule[index] = ITERATION_BATCH_SIZE;
    }
    schedule
}

fn run_iterations(
    state: &mut RootActionState,
    initial_battle: &BattleState,
    player: PlayerName,
    iterations: u32,
    deadline: Instant,
) {
    let log_tree_stats = initial_battle.request_context.logging_options.log_ai_decisions;
    for _ in 0..iterations {
        if Instant::now() >= deadline {
            return;
        }

        let mut battle = player_state::randomize_battle_player(
            initial_battle,
            player.opponent(),
            state.randomize_player_rng.random(),
        );
        battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(&mut battle, player, state.action);

        let mut traversal_stats =
            if log_tree_stats { Some(TreeTraversalAccumulator::default()) } else { None };
        let node = next_evaluation_target(
            &mut battle,
            &mut state.graph,
            state.root,
            traversal_stats.as_mut(),
        );
        let reward = evaluate(&mut battle, player, state);
        back_propagate_rewards(&mut state.graph, player, node, reward);

        match reward.0 {
            r if r > 0.0 => state.wins += 1,
            r if r < 0.0 => state.losses += 1,
            _ => state.draws += 1,
        }
    }
}

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
            LegalActions::Standard { actions } => {
                heuristic_expand_standard_action(battle, player, actions, explored)
            }
            _ => actions.find_missing(explored),
        };

        if let Some(action) = untried_action {
            if let Some(accumulator) = stats.as_mut() {
                accumulator.record_expansion(depth, player, &action);
            }
            return add_child(battle, graph, player, node, action);
        }

        if let Some(accumulator) = stats.as_mut() {
            accumulator.record_selection(depth, player);
        }
        let best = best_child(graph, node, &actions, SelectionMode::Exploration);
        battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(battle, player, best.action);
        node = best.node;
        depth += 1;
    }
    node
}

fn heuristic_expand_standard_action(
    battle: &BattleState,
    player: PlayerName,
    actions: &StandardLegalActions,
    explored: &[BattleAction],
) -> Option<BattleAction> {
    let legal = LegalActions::Standard { actions: actions.clone() };
    let mut scored: Vec<_> = legal
        .all()
        .iter()
        .copied()
        .filter(|action| !explored.contains(action))
        .map(|action| {
            (action, rollout_policy::score_standard_action(battle, player, actions, action))
        })
        .collect();
    scored.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(Ordering::Equal));
    scored.first().map(|(action, _)| *action)
}

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
        .filter(|action| !explored.contains(action))
        .collect();

    if untried.is_empty() {
        return None;
    }

    let assignments = position_assignment::generate(battle, player);
    if assignments.is_empty() {
        return untried.first().copied();
    }

    if untried.contains(&BattleAction::EndTurn) {
        return Some(BattleAction::EndTurn);
    }

    let mut char_scores: Vec<(BattleAction, usize)> = untried
        .iter()
        .filter_map(|action| {
            if let BattleAction::SelectCharacterForPositioning(character_id) = action {
                let count = assignments
                    .iter()
                    .filter(|assignment| {
                        assignment.placements.iter().any(|(id, placement)| {
                            *id == *character_id
                                && matches!(placement, CharacterPlacement::MoveToFrontRank(_))
                        })
                    })
                    .count();
                Some((*action, count))
            } else {
                None
            }
        })
        .collect();
    char_scores.sort_by(|left, right| right.1.cmp(&left.1));
    char_scores.first().map(|(action, _)| *action)
}

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
        .map(|&column| BattleAction::MoveCharacterToFrontRank(character, column))
        .chain(
            attack_column.map(|column| BattleAction::MoveCharacterToFrontRank(character, column)),
        )
        .filter(|action| !explored.contains(action))
        .collect();

    if untried.is_empty() {
        return None;
    }

    let opponent = player.opponent();
    untried.sort_by(|left, right| {
        let left_column = match left {
            BattleAction::MoveCharacterToFrontRank(_, column) => *column,
            _ => 0,
        };
        let right_column = match right {
            BattleAction::MoveCharacterToFrontRank(_, column) => *column,
            _ => 0,
        };
        let left_spark = battle.cards.battlefield(opponent).front[left_column as usize]
            .and_then(|id| card_properties::spark(battle, opponent, id))
            .map_or(0, |spark| spark.0);
        let right_spark = battle.cards.battlefield(opponent).front[right_column as usize]
            .and_then(|id| card_properties::spark(battle, opponent, id))
            .map_or(0, |spark| spark.0);
        right_spark.cmp(&left_spark)
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
    graph.add_edge(parent, child, SearchEdge { action, prior: 0.0 });
    child
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
        .filter(|edge| legal.contains(edge.weight().action, ForPlayer::Agent))
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
            Some(parent) => parent,
            None => break,
        };
    }
}

fn evaluate(
    battle: &mut BattleState,
    maximizing_player: PlayerName,
    state: &mut RootActionState,
) -> OrderedFloat<f64> {
    let mut rollout_actions = 0u32;
    let mut handoffs = 0u32;
    let mut cutoff_pending = false;
    let mut resolution_actions = 0u32;
    let mut previous_active_player = battle.turn.active_player;

    loop {
        if let BattleStatus::GameOver { winner } = battle.status {
            let reward = if winner == Some(maximizing_player) {
                1.0
            } else if winner.is_some() {
                -1.0
            } else {
                0.0
            };
            state.rollout_action_count += rollout_actions;
            return OrderedFloat(reward);
        }

        if cutoff_pending
            && (is_stable_rollout_state(battle) || resolution_actions >= MAX_RESOLUTION_ACTIONS)
        {
            state.cutoff_count += 1;
            state.rollout_action_count += rollout_actions;
            return OrderedFloat(normalize_leaf_score(hybrid_eval::score(
                battle,
                maximizing_player,
            )));
        }

        let Some(player) = legal_actions::next_to_act(battle) else {
            state.rollout_action_count += rollout_actions;
            return OrderedFloat(normalize_leaf_score(hybrid_eval::score(
                battle,
                maximizing_player,
            )));
        };
        let legal = legal_actions::compute(battle, player);
        let Some(choice) = rollout_action(battle, player, &legal) else {
            state.rollout_action_count += rollout_actions;
            return OrderedFloat(normalize_leaf_score(hybrid_eval::score(
                battle,
                maximizing_player,
            )));
        };
        state.pass_suppression_count += u32::from(choice.pass_suppressed);

        if let Some(assignment) = choice.assignment {
            rollout_policy::apply_position_assignment(battle, player, &assignment);
        } else {
            apply_battle_action::execute(battle, player, choice.action);
        }

        rollout_actions += 1;
        if battle.turn.active_player != previous_active_player {
            handoffs += 1;
            previous_active_player = battle.turn.active_player;
        }
        if cutoff_pending {
            resolution_actions += 1;
        } else if rollout_actions >= MAX_ROLLOUT_ACTIONS || handoffs >= MAX_ROLLOUT_HANDOFFS {
            cutoff_pending = true;
        }
    }
}

fn rollout_action(
    battle: &BattleState,
    player: PlayerName,
    legal: &LegalActions,
) -> Option<HybridRolloutChoice> {
    match legal {
        LegalActions::Standard { actions } => {
            rollout_policy::pick_hybrid_rollout_action(battle, player, actions)
        }
        LegalActions::SelectPositioningCharacter { eligible } => {
            heuristic_select_positioning_character(battle, player, eligible).map(|action| {
                HybridRolloutChoice { action, assignment: None, pass_suppressed: false }
            })
        }
        LegalActions::AssignColumn { character, block_targets, attack_column } => {
            heuristic_assign_column(battle, player, *character, block_targets, *attack_column).map(
                |action| HybridRolloutChoice { action, assignment: None, pass_suppressed: false },
            )
        }
        _ => legal.random_action().map(|action| HybridRolloutChoice {
            action,
            assignment: None,
            pass_suppressed: false,
        }),
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
            .filter_map(|id| card_properties::spark(battle, opponent, id))
            .map(|spark| spark.0)
            .max()
            .unwrap_or(0);
        let own_max_spark = eligible
            .iter()
            .filter_map(|id| card_properties::spark(battle, player, id))
            .map(|spark| spark.0)
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
    let own_spark = card_properties::spark(battle, player, character).unwrap_or_default().0;
    let opponent = player.opponent();

    let mut best_block_column = None;
    let mut best_block_spark = 0u32;
    for &column in block_targets {
        if let Some(opp_id) = battle.cards.battlefield(opponent).front[column as usize] {
            let opp_spark = card_properties::spark(battle, opponent, opp_id).unwrap_or_default().0;
            if opp_spark > best_block_spark {
                best_block_spark = opp_spark;
                best_block_column = Some(column);
            }
        }
    }

    if let Some(column) = best_block_column
        && best_block_spark >= own_spark
    {
        return Some(BattleAction::MoveCharacterToFrontRank(character, column));
    }

    if let Some(column) = attack_column {
        return Some(BattleAction::MoveCharacterToFrontRank(character, column));
    }

    if block_targets.is_empty() {
        None
    } else {
        let index = fastrand::usize(..block_targets.len());
        Some(BattleAction::MoveCharacterToFrontRank(character, block_targets[index]))
    }
}

fn is_stable_rollout_state(battle: &BattleState) -> bool {
    if battle.cards.has_stack()
        || battle.stack_priority.is_some()
        || !battle.prompts.is_empty()
        || battle.turn.positioning_started
        || battle.turn.positioning_character.is_some()
    {
        return false;
    }

    legal_actions::next_to_act(battle).is_some_and(|player| {
        matches!(legal_actions::compute(battle, player), LegalActions::Standard { .. })
    })
}

fn normalize_leaf_score(score: f64) -> f64 {
    (score / 2_500.0).tanh().clamp(-1.0, 1.0)
}

fn state_avg_reward(state: &RootActionState) -> f64 {
    if state_visit_count(state) == 0 {
        f64::NEG_INFINITY
    } else {
        state.graph[state.root].total_reward.0 / f64::from(state_visit_count(state))
    }
}

fn state_visit_count(state: &RootActionState) -> u32 {
    state.graph[state.root].visit_count
}

fn write_decision_log(
    battle: &BattleState,
    player: PlayerName,
    budget_ms: u32,
    legal_action_count: usize,
    states: &[RootActionState],
    flat_prior: bool,
    chosen: &RootActionState,
    elapsed: Duration,
) {
    let total_iterations: u32 = states.iter().map(state_visit_count).sum();
    let average_rollout_length = if total_iterations == 0 {
        0.0
    } else {
        f64::from(states.iter().map(|state| state.rollout_action_count).sum::<u32>())
            / f64::from(total_iterations)
    };

    let entry = DecisionLogEntryHybrid {
        average_rollout_length,
        budget_ms,
        chosen_action: format!("{:?}", chosen.action),
        chosen_action_short: chosen.action.battle_action_string(),
        chosen_avg_reward: state_avg_reward(chosen),
        elapsed_ms: elapsed.as_millis(),
        flat_prior,
        game_state: decision_log::build_game_state_snapshot(battle),
        legal_action_count,
        player: hybrid_decision_log::player_string(player),
        rollout_cutoff_count: states.iter().map(|state| state.cutoff_count).sum(),
        rollout_pass_suppression_count: states
            .iter()
            .map(|state| state.pass_suppression_count)
            .sum(),
        root_actions: states
            .iter()
            .map(|state| RootActionLog {
                action: format!("{:?}", state.action),
                action_short: state.action.battle_action_string(),
                avg_reward: if state_visit_count(state) == 0 {
                    0.0
                } else {
                    state.graph[state.root].total_reward.0 / f64::from(state_visit_count(state))
                },
                draws: state.draws,
                heuristic_prior_score: state.heuristic_prior_score,
                iterations_allocated: state_visit_count(state),
                losses: state.losses,
                prior_score: state.prior_score,
                prior_share: state.prior_share,
                wins: state.wins,
            })
            .collect(),
        timestamp: Utc::now().to_rfc3339(),
        total_iterations,
    };
    hybrid_decision_log::write_decision_log(&entry, battle);
}
