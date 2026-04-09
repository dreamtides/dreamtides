use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use battle_mutations::actions::apply_battle_action;
use battle_mutations::player_mutations::player_state;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use core_data::types::PlayerName;
use rand::Rng;
use rayon::prelude::*;

use crate::candidate_generation_v2::{self, CandidateExecution, RootCandidate};
use crate::decision_log::{self, StrategicCandidateLog, WriteDecisionLog};
use crate::{evaluation, state_key};

const INITIAL_SAMPLE_ROUNDS: usize = 2;
const MAX_ROOT_CANDIDATES: usize = 12;
const MAX_REPLY_CANDIDATES: usize = 6;
const MAX_TACTICAL_ACTIONS: usize = 8;
const ROOT_LOOKAHEAD_PLIES: u8 = 2;
const TACTICAL_DEPTH: u8 = 5;
const SAFETY_ACTION_CAP: usize = 128;
const LOOP_PENALTY: f64 = 20.0;

pub struct SearchResult {
    pub action: BattleAction,
    pub pending_actions: Vec<BattleAction>,
}

pub fn search(initial_battle: &BattleState, player: PlayerName, budget_ms: u32) -> SearchResult {
    let start = Instant::now();
    let deadline = start + Duration::from_millis(u64::from(budget_ms));
    let planning_battle = player_state::randomize_battle_player(
        initial_battle,
        player.opponent(),
        rand::rng().random(),
    );

    if is_tactical_state(&planning_battle) {
        let (action, score, transposition_hits) =
            choose_tactical_action(initial_battle, player, deadline);
        decision_log::write_decision_log(WriteDecisionLog {
            battle: initial_battle,
            candidates: Vec::new(),
            chosen_action: action,
            chosen_score: score,
            elapsed: start.elapsed(),
            evaluation_features: evaluation::breakdown(&planning_battle, player).features,
            fallback_used: false,
            mode: "tactical-v2",
            player,
            transposition_hits,
        });
        return SearchResult { action, pending_actions: Vec::new() };
    }

    let mut candidates = distinct_root_candidates(&planning_battle, player, deadline);
    if candidates.is_empty() {
        let fallback = fallback_action(&planning_battle, player);
        decision_log::write_decision_log(WriteDecisionLog {
            battle: initial_battle,
            candidates: Vec::new(),
            chosen_action: fallback,
            chosen_score: evaluation::score(&planning_battle, player),
            elapsed: start.elapsed(),
            evaluation_features: evaluation::breakdown(&planning_battle, player).features,
            fallback_used: true,
            mode: "stable-v2",
            player,
            transposition_hits: 0,
        });
        return SearchResult { action: fallback, pending_actions: Vec::new() };
    }

    let mut aggregated: Vec<_> = candidates
        .drain(..)
        .map(|candidate| AggregatedCandidate {
            aggregate_score: candidate.heuristic_score,
            heuristic_score: candidate.heuristic_score,
            candidate,
            samples: Vec::new(),
        })
        .collect();
    let mut transposition_hits = 0usize;

    for _ in 0..INITIAL_SAMPLE_ROUNDS {
        if Instant::now() >= deadline {
            break;
        }

        let seeds: Vec<_> = (0..aggregated.len()).map(|_| rand::rng().random::<u64>()).collect();
        let scores: Vec<_> = aggregated
            .par_iter()
            .zip(seeds.par_iter())
            .map(|(candidate, seed)| {
                score_candidate_sample(
                    initial_battle,
                    player,
                    &candidate.candidate,
                    *seed,
                    deadline,
                )
            })
            .collect();

        for (index, (score, hits)) in scores.into_iter().enumerate() {
            aggregated[index].samples.push(score);
            aggregated[index].aggregate_score = aggregate_scores(&aggregated[index].samples);
            transposition_hits += hits;
        }

        aggregated.sort_by(|left, right| right.aggregate_score.total_cmp(&left.aggregate_score));
        aggregated.truncate(8);
    }

    while aggregated.len() > 4 {
        aggregated.pop();
    }

    if Instant::now() < deadline {
        let seeds: Vec<_> = (0..aggregated.len()).map(|_| rand::rng().random::<u64>()).collect();
        let scores: Vec<_> = aggregated
            .par_iter()
            .zip(seeds.par_iter())
            .map(|(candidate, seed)| {
                score_candidate_sample(
                    initial_battle,
                    player,
                    &candidate.candidate,
                    *seed,
                    deadline,
                )
            })
            .collect();

        for (index, (score, hits)) in scores.into_iter().enumerate() {
            aggregated[index].samples.push(score);
            aggregated[index].aggregate_score = aggregate_scores(&aggregated[index].samples);
            transposition_hits += hits;
        }
    }

    aggregated.sort_by(|left, right| right.aggregate_score.total_cmp(&left.aggregate_score));
    let best = aggregated.first().cloned().unwrap_or_else(|| AggregatedCandidate {
        candidate: RootCandidate {
            action: fallback_action(&planning_battle, player),
            description: "fallback".to_string(),
            execution: CandidateExecution::Action(fallback_action(&planning_battle, player)),
            heuristic_score: evaluation::score(&planning_battle, player),
            pending_actions: Vec::new(),
        },
        aggregate_score: evaluation::score(&planning_battle, player),
        heuristic_score: evaluation::score(&planning_battle, player),
        samples: Vec::new(),
    });
    let candidate_logs = aggregated
        .into_iter()
        .map(|candidate| StrategicCandidateLog {
            description: candidate.candidate.description.clone(),
            first_action: format!("{:?}", candidate.candidate.action),
            first_action_short: candidate.candidate.action.battle_action_string(),
            heuristic_score: candidate.heuristic_score,
            aggregate_score: candidate.aggregate_score,
            samples: candidate.samples.len(),
        })
        .collect();

    decision_log::write_decision_log(WriteDecisionLog {
        battle: initial_battle,
        candidates: candidate_logs,
        chosen_action: best.candidate.action,
        chosen_score: best.aggregate_score,
        elapsed: start.elapsed(),
        evaluation_features: evaluation::breakdown(&planning_battle, player).features,
        fallback_used: false,
        mode: "stable-v2",
        player,
        transposition_hits,
    });
    SearchResult { action: best.candidate.action, pending_actions: best.candidate.pending_actions }
}

pub fn is_tactical_state(battle: &BattleState) -> bool {
    battle.cards.has_stack()
        || battle.stack_priority.is_some()
        || !battle.prompts.is_empty()
        || battle.turn.positioning_started
        || battle.turn.positioning_character.is_some()
        || legal_actions::next_to_act(battle).is_some_and(|player| {
            !matches!(legal_actions::compute(battle, player), LegalActions::Standard { .. })
        })
}

struct SearchContext {
    deadline: Instant,
    root_player: PlayerName,
    strategic_memo: HashMap<(u64, u8), f64>,
    tactical_memo: HashMap<(u64, u8, PlayerName), f64>,
    tactical_visited: HashSet<u64>,
    strategic_visited: HashSet<u64>,
    transposition_hits: usize,
}

#[derive(Clone)]
struct AggregatedCandidate {
    candidate: RootCandidate,
    aggregate_score: f64,
    heuristic_score: f64,
    samples: Vec<f64>,
}

fn distinct_root_candidates(
    battle: &BattleState,
    player: PlayerName,
    deadline: Instant,
) -> Vec<RootCandidate> {
    let mut by_state = HashMap::new();

    for candidate in candidate_generation_v2::generate_root_candidates(battle, player) {
        if Instant::now() >= deadline {
            break;
        }

        let mut next_battle = battle.logical_clone();
        next_battle.request_context.logging_options.enable_action_legality_check = false;
        candidate_generation_v2::apply_root_candidate(&mut next_battle, player, &candidate);
        let mut context = SearchContext {
            deadline,
            root_player: player,
            strategic_memo: HashMap::new(),
            tactical_memo: HashMap::new(),
            tactical_visited: HashSet::new(),
            strategic_visited: HashSet::new(),
            transposition_hits: 0,
        };
        advance_to_next_stable_state(&mut next_battle, &mut context);
        let stable_key = state_key::key(&next_battle);
        let stable_score = evaluation::score(&next_battle, player);
        let adjusted_candidate = RootCandidate { heuristic_score: stable_score, ..candidate };
        by_state
            .entry(stable_key)
            .and_modify(|existing: &mut RootCandidate| {
                if adjusted_candidate.heuristic_score > existing.heuristic_score {
                    *existing = adjusted_candidate.clone();
                }
            })
            .or_insert(adjusted_candidate);
    }

    let mut result: Vec<_> = by_state.into_values().collect();
    result.sort_by(|left, right| right.heuristic_score.total_cmp(&left.heuristic_score));
    result.truncate(MAX_ROOT_CANDIDATES);
    result
}

fn advance_to_next_stable_state(battle: &mut BattleState, context: &mut SearchContext) {
    let mut actions_taken = 0usize;
    while actions_taken < SAFETY_ACTION_CAP && Instant::now() < context.deadline {
        if matches!(battle.status, BattleStatus::GameOver { .. }) {
            return;
        }
        if is_stable_state(battle) {
            return;
        }
        let Some(actor) = legal_actions::next_to_act(battle) else {
            return;
        };
        let action = choose_resolving_action(battle, actor, context);
        battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(battle, actor, action);
        actions_taken += 1;
    }
}

fn choose_resolving_action(
    battle: &BattleState,
    actor: PlayerName,
    context: &mut SearchContext,
) -> BattleAction {
    let legal = legal_actions::compute(battle, actor);
    let actions = prioritize_tactical_actions(battle, actor, &legal);
    let mut best_action =
        actions.first().copied().unwrap_or_else(|| fallback_action(battle, actor));
    let mut best_score =
        if actor == context.root_player { f64::NEG_INFINITY } else { f64::INFINITY };

    for action in actions.into_iter().take(MAX_TACTICAL_ACTIONS) {
        if Instant::now() >= context.deadline {
            break;
        }
        let mut next_battle = battle.logical_clone();
        next_battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(&mut next_battle, actor, action);
        let score = tactical_value(&next_battle, TACTICAL_DEPTH, context);
        if actor == context.root_player {
            if score > best_score {
                best_score = score;
                best_action = action;
            }
        } else if score < best_score {
            best_score = score;
            best_action = action;
        }
    }

    best_action
}

fn choose_tactical_action(
    initial_battle: &BattleState,
    player: PlayerName,
    deadline: Instant,
) -> (BattleAction, f64, usize) {
    let legal = legal_actions::compute(initial_battle, player);
    let actions = prioritize_tactical_actions(initial_battle, player, &legal);
    let mut best_action =
        actions.first().copied().unwrap_or_else(|| fallback_action(initial_battle, player));
    let mut best_score = f64::NEG_INFINITY;
    let mut transposition_hits = 0usize;

    for action in actions.into_iter().take(MAX_TACTICAL_ACTIONS) {
        if Instant::now() >= deadline {
            break;
        }

        let mut samples = Vec::new();
        for _ in 0..3 {
            if Instant::now() >= deadline {
                break;
            }
            let mut battle = player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            battle.request_context.logging_options.enable_action_legality_check = false;
            apply_battle_action::execute(&mut battle, player, action);
            let mut context = SearchContext {
                deadline,
                root_player: player,
                strategic_memo: HashMap::new(),
                tactical_memo: HashMap::new(),
                tactical_visited: HashSet::new(),
                strategic_visited: HashSet::new(),
                transposition_hits: 0,
            };
            samples.push(tactical_value(&battle, TACTICAL_DEPTH, &mut context));
            transposition_hits += context.transposition_hits;
        }

        let score = aggregate_scores(&samples);
        if score > best_score {
            best_score = score;
            best_action = action;
        }
    }

    (best_action, best_score, transposition_hits)
}

fn score_candidate_sample(
    initial_battle: &BattleState,
    player: PlayerName,
    candidate: &RootCandidate,
    seed: u64,
    deadline: Instant,
) -> (f64, usize) {
    let mut battle = player_state::randomize_battle_player(initial_battle, player.opponent(), seed);
    battle.request_context.logging_options.enable_action_legality_check = false;
    candidate_generation_v2::apply_root_candidate(&mut battle, player, candidate);
    let mut context = SearchContext {
        deadline,
        root_player: player,
        strategic_memo: HashMap::new(),
        tactical_memo: HashMap::new(),
        tactical_visited: HashSet::new(),
        strategic_visited: HashSet::new(),
        transposition_hits: 0,
    };
    advance_to_next_stable_state(&mut battle, &mut context);
    let score = strategic_value(&battle, ROOT_LOOKAHEAD_PLIES, &mut context);
    (score, context.transposition_hits)
}

fn strategic_value(battle: &BattleState, depth: u8, context: &mut SearchContext) -> f64 {
    if Instant::now() >= context.deadline {
        return evaluation::score(battle, context.root_player);
    }
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return evaluation::score(battle, context.root_player);
    }
    if depth == 0 {
        return evaluation::score(battle, context.root_player);
    }

    if is_tactical_state(battle) {
        let mut resolved = battle.logical_clone();
        advance_to_next_stable_state(&mut resolved, context);
        return strategic_value(&resolved, depth, context);
    }

    let state = state_key::key(battle);
    if !context.strategic_visited.insert(state) {
        return evaluation::score(battle, context.root_player) - LOOP_PENALTY;
    }
    if let Some(score) = context.strategic_memo.get(&(state, depth)) {
        context.transposition_hits += 1;
        context.strategic_visited.remove(&state);
        return *score;
    }

    let Some(actor) = legal_actions::next_to_act(battle) else {
        context.strategic_visited.remove(&state);
        return evaluation::score(battle, context.root_player);
    };
    let candidates = distinct_root_candidates(battle, actor, context.deadline);
    if candidates.is_empty() {
        context.strategic_visited.remove(&state);
        return evaluation::score(battle, context.root_player);
    }

    let mut best = if actor == context.root_player { f64::NEG_INFINITY } else { f64::INFINITY };
    for candidate in candidates.into_iter().take(MAX_REPLY_CANDIDATES) {
        if Instant::now() >= context.deadline {
            break;
        }
        let mut next_battle = battle.logical_clone();
        next_battle.request_context.logging_options.enable_action_legality_check = false;
        candidate_generation_v2::apply_root_candidate(&mut next_battle, actor, &candidate);
        advance_to_next_stable_state(&mut next_battle, context);
        let score = strategic_value(&next_battle, depth - 1, context);
        if actor == context.root_player {
            best = best.max(score);
        } else {
            best = best.min(score);
        }
    }

    let result =
        if best.is_finite() { best } else { evaluation::score(battle, context.root_player) };
    context.strategic_memo.insert((state, depth), result);
    context.strategic_visited.remove(&state);
    result
}

fn tactical_value(battle: &BattleState, depth: u8, context: &mut SearchContext) -> f64 {
    if Instant::now() >= context.deadline {
        return evaluation::score(battle, context.root_player);
    }
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return evaluation::score(battle, context.root_player);
    }
    if depth == 0 || is_stable_state(battle) {
        return evaluation::score(battle, context.root_player);
    }

    let state = state_key::key(battle);
    if !context.tactical_visited.insert(state) {
        return evaluation::score(battle, context.root_player) - LOOP_PENALTY;
    }

    let Some(actor) = legal_actions::next_to_act(battle) else {
        context.tactical_visited.remove(&state);
        return evaluation::score(battle, context.root_player);
    };
    if let Some(score) = context.tactical_memo.get(&(state, depth, actor)) {
        context.transposition_hits += 1;
        context.tactical_visited.remove(&state);
        return *score;
    }

    let legal = legal_actions::compute(battle, actor);
    let actions = prioritize_tactical_actions(battle, actor, &legal);
    let mut best = if actor == context.root_player { f64::NEG_INFINITY } else { f64::INFINITY };

    for action in actions.into_iter().take(MAX_TACTICAL_ACTIONS) {
        if Instant::now() >= context.deadline {
            break;
        }
        let mut next_battle = battle.logical_clone();
        next_battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(&mut next_battle, actor, action);
        let score = tactical_value(&next_battle, depth - 1, context);
        if actor == context.root_player {
            best = best.max(score);
        } else {
            best = best.min(score);
        }
    }

    let result =
        if best.is_finite() { best } else { evaluation::score(battle, context.root_player) };
    context.tactical_memo.insert((state, depth, actor), result);
    context.tactical_visited.remove(&state);
    result
}

fn prioritize_tactical_actions(
    battle: &BattleState,
    actor: PlayerName,
    legal: &LegalActions,
) -> Vec<BattleAction> {
    let mut ranked: Vec<_> = legal
        .all()
        .into_iter()
        .map(|action| {
            let mut next_battle = battle.logical_clone();
            next_battle.request_context.logging_options.enable_action_legality_check = false;
            apply_battle_action::execute(&mut next_battle, actor, action);
            (evaluation::score(&next_battle, actor), action_priority(action), action)
        })
        .collect();
    ranked.sort_by(|left, right| (right.0 + right.1).total_cmp(&(left.0 + left.1)));
    ranked.into_iter().map(|(_, _, action)| action).collect()
}

fn is_stable_state(battle: &BattleState) -> bool {
    !is_tactical_state(battle)
        && legal_actions::next_to_act(battle).is_some_and(|player| {
            matches!(legal_actions::compute(battle, player), LegalActions::Standard { .. })
        })
}

fn fallback_action(battle: &BattleState, player: PlayerName) -> BattleAction {
    legal_actions::compute(battle, player).all().first().copied().unwrap_or(BattleAction::EndTurn)
}

fn aggregate_scores(scores: &[f64]) -> f64 {
    if scores.is_empty() {
        return f64::NEG_INFINITY;
    }
    let mean = scores.iter().sum::<f64>() / scores.len() as f64;
    let mut ordered = scores.to_vec();
    ordered.sort_by(f64::total_cmp);
    let lower_quartile_count = ordered.len().div_ceil(4);
    let lower_quartile_mean =
        ordered.iter().take(lower_quartile_count).sum::<f64>() / lower_quartile_count as f64;
    mean * 0.7 + lower_quartile_mean * 0.3
}

fn action_priority(action: BattleAction) -> f64 {
    match action {
        BattleAction::EndTurn => -12.0,
        BattleAction::PassPriority => -8.0,
        _ => 0.0,
    }
}
