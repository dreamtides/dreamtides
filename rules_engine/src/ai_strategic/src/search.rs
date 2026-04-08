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
use tracing::debug;

use crate::candidate_generation::{self, CandidatePlan};
use crate::decision_log::{self, StrategicCandidateLog, WriteDecisionLog};
use crate::{evaluation, state_key};

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

    let (mode, chosen, chosen_score, candidates, transposition_hits, fallback_used) =
        if is_tactical_state(&planning_battle) {
            let (action, score, transpositions, fallback) =
                choose_tactical_action(initial_battle, player, deadline);
            let features = evaluation::breakdown(&planning_battle, player).features;
            decision_log::write_decision_log(WriteDecisionLog {
                battle: initial_battle,
                candidates: Vec::new(),
                chosen_action: action,
                chosen_score: score,
                elapsed: start.elapsed(),
                evaluation_features: features,
                fallback_used: fallback,
                mode: "tactical",
                player,
                transposition_hits: transpositions,
            });
            return SearchResult { action, pending_actions: Vec::new() };
        } else {
            choose_proactive_action(initial_battle, &planning_battle, player, deadline)
        };

    let breakdown = evaluation::breakdown(&planning_battle, player);
    decision_log::write_decision_log(WriteDecisionLog {
        battle: initial_battle,
        candidates,
        chosen_action: chosen.action,
        chosen_score,
        elapsed: start.elapsed(),
        evaluation_features: breakdown.features,
        fallback_used,
        mode,
        player,
        transposition_hits,
    });
    SearchResult { action: chosen.action, pending_actions: chosen.pending_actions }
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

struct TacticalContext {
    deadline: Instant,
    memo: HashMap<(u64, u8, PlayerName), f64>,
    root_player: PlayerName,
    transposition_hits: usize,
    visited: HashSet<u64>,
}

#[derive(Clone)]
struct AggregatedCandidate {
    action: BattleAction,
    aggregate_score: f64,
    description: String,
    heuristic_score: f64,
    pending_actions: Vec<BattleAction>,
    plan: CandidatePlan,
    samples: Vec<f64>,
}

fn choose_proactive_action(
    initial_battle: &BattleState,
    planning_battle: &BattleState,
    player: PlayerName,
    deadline: Instant,
) -> (&'static str, SearchResult, f64, Vec<StrategicCandidateLog>, usize, bool) {
    let mut candidates =
        candidate_generation::generate_root_candidates(planning_battle, player, 16, 8, 5, deadline);
    if candidates.is_empty() {
        let fallback = fallback_action(planning_battle, player);
        return (
            "proactive",
            SearchResult { action: fallback, pending_actions: Vec::new() },
            evaluation::score(planning_battle, player),
            Vec::new(),
            0,
            true,
        );
    }

    let mut aggregated: Vec<_> = candidates
        .drain(..)
        .map(|candidate| AggregatedCandidate {
            action: candidate.first_action,
            aggregate_score: candidate.heuristic_score,
            description: candidate.description.clone(),
            heuristic_score: candidate.heuristic_score,
            pending_actions: candidate.pending_actions.clone(),
            plan: candidate,
            samples: Vec::new(),
        })
        .collect();

    let prune_schedule = [(0usize, 8usize), (1usize, 4usize), (2usize, 2usize)];
    let mut transposition_hits = 0usize;

    for round in 0..4 {
        if Instant::now() >= deadline {
            break;
        }

        let seeds: Vec<u64> = (0..aggregated.len()).map(|_| rand::rng().random::<u64>()).collect();
        let scores: Vec<_> = aggregated
            .par_iter()
            .zip(seeds.par_iter())
            .map(|(candidate, seed)| {
                score_plan_sample(initial_battle, player, &candidate.plan, *seed, deadline)
            })
            .collect();

        for (index, (score, hits)) in scores.into_iter().enumerate() {
            aggregated[index].samples.push(score);
            aggregated[index].aggregate_score = aggregate_scores(&aggregated[index].samples);
            transposition_hits += hits;
        }

        aggregated.sort_by(|left, right| right.aggregate_score.total_cmp(&left.aggregate_score));

        if let Some((schedule_round, keep_count)) =
            prune_schedule.iter().find(|(schedule_round, _)| *schedule_round == round)
        {
            let _ = schedule_round;
            aggregated.truncate(*keep_count);
        }
    }

    let best = aggregated.first().cloned().unwrap_or_else(|| AggregatedCandidate {
        action: fallback_action(planning_battle, player),
        aggregate_score: evaluation::score(planning_battle, player),
        description: "fallback".to_string(),
        heuristic_score: evaluation::score(planning_battle, player),
        pending_actions: Vec::new(),
        plan: CandidatePlan {
            description: "fallback".to_string(),
            first_action: fallback_action(planning_battle, player),
            heuristic_score: evaluation::score(planning_battle, player),
            pending_actions: Vec::new(),
            steps: Vec::new(),
        },
        samples: Vec::new(),
    });
    let fallback_used = best.description == "fallback";
    let candidate_logs = aggregated
        .into_iter()
        .map(|candidate| StrategicCandidateLog {
            description: candidate.description.clone(),
            first_action: format!("{:?}", candidate.action),
            first_action_short: candidate.action.battle_action_string(),
            heuristic_score: candidate.heuristic_score,
            aggregate_score: candidate.aggregate_score,
            samples: candidate.samples.len(),
        })
        .collect();

    (
        "proactive",
        SearchResult { action: best.action, pending_actions: best.pending_actions },
        best.aggregate_score,
        candidate_logs,
        transposition_hits,
        fallback_used,
    )
}

fn choose_tactical_action(
    initial_battle: &BattleState,
    player: PlayerName,
    deadline: Instant,
) -> (BattleAction, f64, usize, bool) {
    let legal = legal_actions::compute(initial_battle, player);
    let actions = prioritize_tactical_actions(initial_battle, player, &legal);
    if actions.is_empty() {
        let fallback = fallback_action(initial_battle, player);
        return (fallback, evaluation::score(initial_battle, player), 0, true);
    }

    let mut best_action = actions[0];
    let mut best_score = f64::NEG_INFINITY;
    let mut total_transposition_hits = 0usize;

    for action in actions {
        if Instant::now() >= deadline {
            break;
        }

        let mut sample_total = 0.0;
        let mut samples = 0usize;
        while samples < 3 && Instant::now() < deadline {
            let mut battle = player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                rand::rng().random(),
            );
            battle.request_context.logging_options.enable_action_legality_check = false;
            apply_battle_action::execute(&mut battle, player, action);
            let mut context = TacticalContext {
                deadline,
                memo: HashMap::new(),
                root_player: player,
                transposition_hits: 0,
                visited: HashSet::new(),
            };
            sample_total += tactical_value(&battle, 5, &mut context);
            total_transposition_hits += context.transposition_hits;
            samples += 1;
        }

        let score = if samples > 0 { sample_total / samples as f64 } else { f64::NEG_INFINITY };
        if score > best_score {
            best_score = score;
            best_action = action;
        }
    }

    (best_action, best_score, total_transposition_hits, false)
}

fn score_plan_sample(
    initial_battle: &BattleState,
    player: PlayerName,
    plan: &CandidatePlan,
    seed: u64,
    deadline: Instant,
) -> (f64, usize) {
    let mut battle = player_state::randomize_battle_player(initial_battle, player.opponent(), seed);
    battle.request_context.logging_options.enable_action_legality_check = false;
    candidate_generation::apply_candidate_plan(&mut battle, player, plan);
    let mut context = TacticalContext {
        deadline,
        memo: HashMap::new(),
        root_player: player,
        transposition_hits: 0,
        visited: HashSet::new(),
    };
    let score = continuation_value(&battle, 1, &mut context);
    (score, context.transposition_hits)
}

fn continuation_value(battle: &BattleState, depth: u8, context: &mut TacticalContext) -> f64 {
    if Instant::now() >= context.deadline {
        return evaluation::score(battle, context.root_player);
    }

    if let BattleStatus::GameOver { .. } = battle.status {
        return evaluation::score(battle, context.root_player);
    }

    if is_tactical_state(battle) {
        return tactical_value(battle, 4, context);
    }

    let Some(actor) = legal_actions::next_to_act(battle) else {
        return evaluation::score(battle, context.root_player);
    };
    if depth == 0 {
        return evaluation::score(battle, context.root_player);
    }

    let candidates =
        candidate_generation::generate_root_candidates(battle, actor, 8, 4, 3, context.deadline);
    if candidates.is_empty() {
        return evaluation::score(battle, context.root_player);
    }

    let relevant = candidates.into_iter().take(4);
    if actor == context.root_player {
        relevant
            .map(|candidate| {
                let mut next_battle = battle.logical_clone();
                next_battle.request_context.logging_options.enable_action_legality_check = false;
                candidate_generation::apply_candidate_plan(&mut next_battle, actor, &candidate);
                continuation_value(&next_battle, depth - 1, context)
            })
            .max_by(f64::total_cmp)
            .unwrap_or_else(|| evaluation::score(battle, context.root_player))
    } else {
        relevant
            .map(|candidate| {
                let mut next_battle = battle.logical_clone();
                next_battle.request_context.logging_options.enable_action_legality_check = false;
                candidate_generation::apply_candidate_plan(&mut next_battle, actor, &candidate);
                continuation_value(&next_battle, depth - 1, context)
            })
            .min_by(f64::total_cmp)
            .unwrap_or_else(|| evaluation::score(battle, context.root_player))
    }
}

fn tactical_value(battle: &BattleState, depth: u8, context: &mut TacticalContext) -> f64 {
    if Instant::now() >= context.deadline {
        return evaluation::score(battle, context.root_player);
    }

    if let BattleStatus::GameOver { .. } = battle.status {
        return evaluation::score(battle, context.root_player);
    }

    let state_key = state_key::key(battle);
    if !context.visited.insert(state_key) {
        return evaluation::score(battle, context.root_player) - 15.0;
    }

    let Some(actor) = legal_actions::next_to_act(battle) else {
        context.visited.remove(&state_key);
        return evaluation::score(battle, context.root_player);
    };

    if let Some(cached) = context.memo.get(&(state_key, depth, actor)) {
        context.transposition_hits += 1;
        context.visited.remove(&state_key);
        return *cached;
    }

    let legal = legal_actions::compute(battle, actor);
    let actions = prioritize_tactical_actions(battle, actor, &legal);
    if actions.is_empty() {
        context.visited.remove(&state_key);
        return evaluation::score(battle, context.root_player);
    }

    let mut best = if actor == context.root_player { f64::NEG_INFINITY } else { f64::INFINITY };
    let mut explored_any = false;

    for action in actions.into_iter().take(8) {
        let mut next_battle = battle.logical_clone();
        next_battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(&mut next_battle, actor, action);
        let next_depth = depth.saturating_sub(1);
        let score = if depth == 0 && !is_tactical_state(&next_battle) {
            evaluation::score(&next_battle, context.root_player)
        } else {
            tactical_value(&next_battle, next_depth, context)
        };
        explored_any = true;

        if actor == context.root_player {
            best = best.max(score);
        } else {
            best = best.min(score);
        }
    }

    let result = if explored_any { best } else { evaluation::score(battle, context.root_player) };
    context.memo.insert((state_key, depth, actor), result);
    context.visited.remove(&state_key);
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
            (
                evaluation::score(&next_battle, actor),
                if action == BattleAction::PassPriority || action == BattleAction::EndTurn {
                    -5.0
                } else {
                    0.0
                },
                action,
            )
        })
        .collect();
    ranked.sort_by(|left, right| (right.0 + right.1).total_cmp(&(left.0 + left.1)));
    ranked.into_iter().map(|(_, _, action)| action).collect()
}

fn fallback_action(battle: &BattleState, player: PlayerName) -> BattleAction {
    let legal = legal_actions::compute(battle, player);
    let prioritized = prioritize_tactical_actions(battle, player, &legal);
    prioritized.first().copied().unwrap_or_else(|| {
        debug!("Falling back to EndTurn for StrategicV1");
        BattleAction::EndTurn
    })
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
