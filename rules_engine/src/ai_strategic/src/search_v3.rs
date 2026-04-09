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

use crate::candidate_generation_v3::{self, RootCandidate};
use crate::decision_log_v3::{self, DecisionCandidateV3, WriteDecisionLogV3};
use crate::model::ModelPair;
use crate::{feature_extraction, model, state_key};

const EXPLORATION_SLOTS: usize = 2;
const MAX_KEEP_CANDIDATES: usize = 8;
const MIN_STABLE_THINK_MS: u64 = 750;
const STRATEGIC_DEPTH: u8 = 2;
const TACTICAL_DEPTH: u8 = 5;
const LOOP_PENALTY: f64 = 25.0;
const MAX_TACTICAL_ACTIONS: usize = 8;
const SAFETY_ACTION_CAP: usize = 128;

pub struct SearchResult {
    pub action: BattleAction,
    pub pending_actions: Vec<BattleAction>,
}

pub fn search(
    initial_battle: &BattleState,
    player: PlayerName,
    budget_ms: u32,
) -> Option<SearchResult> {
    let models = model::load_models()?;
    let start = Instant::now();
    let deadline = start + Duration::from_millis(u64::from(budget_ms));
    let planning_battle = player_state::randomize_battle_player(
        initial_battle,
        player.opponent(),
        rand::rng().random(),
    );

    if is_tactical_state(&planning_battle) {
        let action = choose_tactical_action(&planning_battle, player, deadline, player);
        decision_log_v3::write_decision_log(WriteDecisionLogV3 {
            battle: initial_battle,
            candidates: Vec::new(),
            chosen_action: action,
            chosen_score: 0.0,
            elapsed: start.elapsed(),
            fallback_used: false,
            legal_action_count: legal_actions::compute(initial_battle, player).len(),
            min_think_floor_hit: false,
            mode: "tactical-v3",
            player,
            strategic_depth: 0,
        });
        return Some(SearchResult { action, pending_actions: Vec::new() });
    }

    let legal_action_count = legal_actions::compute(initial_battle, player).len();
    let mut context = SearchContext {
        deadline,
        models,
        root_player: player,
        strategic_visited: HashSet::new(),
        strategic_memo: HashMap::new(),
    };
    let (best, mut candidates) =
        choose_stable_action(&planning_battle, STRATEGIC_DEPTH, &mut context);

    let mut min_think_floor_hit = false;
    while legal_action_count > 3
        && start.elapsed() < Duration::from_millis(MIN_STABLE_THINK_MS)
        && Instant::now() < deadline
    {
        min_think_floor_hit = true;
        let refreshed = rescore_candidates(&planning_battle, STRATEGIC_DEPTH, &mut context);
        if !refreshed.is_empty() {
            candidates = refreshed;
        } else {
            break;
        }
    }

    let chosen = candidates
        .iter()
        .max_by(|left, right| left.combined_score.total_cmp(&right.combined_score))
        .cloned()
        .unwrap_or(best);
    decision_log_v3::write_decision_log(WriteDecisionLogV3 {
        battle: initial_battle,
        candidates: candidates
            .iter()
            .map(|candidate| DecisionCandidateV3 {
                description: candidate.description.clone(),
                first_action_short: candidate.action.battle_action_string(),
                policy_score: candidate.policy_score,
                value_score: candidate.value_score,
                combined_score: candidate.combined_score,
            })
            .collect(),
        chosen_action: chosen.action,
        chosen_score: chosen.combined_score,
        elapsed: start.elapsed(),
        fallback_used: false,
        legal_action_count,
        min_think_floor_hit,
        mode: "stable-v3",
        player,
        strategic_depth: STRATEGIC_DEPTH,
    });
    Some(SearchResult { action: chosen.action, pending_actions: chosen.pending_actions })
}

pub fn is_available() -> bool {
    model::load_models().is_some()
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

struct SearchContext<'a> {
    deadline: Instant,
    models: ModelPair<'a>,
    root_player: PlayerName,
    strategic_visited: HashSet<u64>,
    strategic_memo: HashMap<(u64, u8, PlayerName), f64>,
}

#[derive(Clone)]
struct ScoredCandidate {
    action: BattleAction,
    combined_score: f64,
    description: String,
    pending_actions: Vec<BattleAction>,
    policy_score: f64,
    value_score: f64,
}

fn choose_stable_action(
    battle: &BattleState,
    depth: u8,
    context: &mut SearchContext<'_>,
) -> (ScoredCandidate, Vec<ScoredCandidate>) {
    let candidates = rescore_candidates(battle, depth, context);
    let best = candidates
        .first()
        .cloned()
        .unwrap_or_else(|| fallback_candidate(battle, context.root_player));
    (best, candidates)
}

fn rescore_candidates(
    battle: &BattleState,
    depth: u8,
    context: &mut SearchContext<'_>,
) -> Vec<ScoredCandidate> {
    let Some(actor) = legal_actions::next_to_act(battle) else {
        return Vec::new();
    };
    let state_features = feature_extraction::extract_state_features(battle, actor);
    let legal_action_count = legal_actions::compute(battle, actor).len();
    let mut candidates = candidate_generation_v3::generate_candidates(battle, actor);

    for candidate in &mut candidates {
        let merged = feature_extraction::merge_policy_features(
            &state_features,
            &candidate.action_features,
            legal_action_count,
        );
        candidate.policy_score = context.models.policy.score(&merged);
    }
    candidates.sort_by(|left, right| right.policy_score.total_cmp(&left.policy_score));

    let kept = prune_candidates(candidates);
    let mut scored = Vec::new();
    for candidate in kept {
        if Instant::now() >= context.deadline {
            break;
        }
        let mut next_battle = battle.logical_clone();
        next_battle.request_context.logging_options.enable_action_legality_check = false;
        candidate_generation_v3::apply_candidate(&mut next_battle, actor, &candidate);
        advance_to_next_stable_state(&mut next_battle, context);
        let value_score = strategic_value(&next_battle, depth.saturating_sub(1), context, actor);
        scored.push(ScoredCandidate {
            action: candidate.action,
            combined_score: candidate.policy_score * 0.35 + value_score * 0.65,
            description: candidate.description,
            pending_actions: candidate.pending_actions,
            policy_score: candidate.policy_score,
            value_score,
        });
    }
    scored.sort_by(|left, right| right.combined_score.total_cmp(&left.combined_score));
    scored
}

fn strategic_value(
    battle: &BattleState,
    depth: u8,
    context: &mut SearchContext<'_>,
    perspective: PlayerName,
) -> f64 {
    if Instant::now() >= context.deadline || matches!(battle.status, BattleStatus::GameOver { .. })
    {
        return state_value(battle, perspective, context);
    }
    if is_tactical_state(battle) {
        let mut resolved = battle.logical_clone();
        advance_to_next_stable_state(&mut resolved, context);
        return strategic_value(&resolved, depth, context, perspective);
    }
    if depth == 0 {
        return state_value(battle, perspective, context);
    }

    let Some(actor) = legal_actions::next_to_act(battle) else {
        return state_value(battle, perspective, context);
    };
    let state = state_key::key(battle);
    if !context.strategic_visited.insert(state) {
        return state_value(battle, perspective, context) - LOOP_PENALTY;
    }
    if let Some(score) = context.strategic_memo.get(&(state, depth, perspective)) {
        context.strategic_visited.remove(&state);
        return *score;
    }

    let candidates = rescore_candidates(battle, depth, context);
    let mut best = if actor == perspective { f64::NEG_INFINITY } else { f64::INFINITY };
    for candidate in candidates.into_iter().take(MAX_KEEP_CANDIDATES) {
        if actor == perspective {
            best = best.max(candidate.combined_score);
        } else {
            best = best.min(candidate.combined_score);
        }
    }

    let result = if best.is_finite() { best } else { state_value(battle, perspective, context) };
    context.strategic_memo.insert((state, depth, perspective), result);
    context.strategic_visited.remove(&state);
    result
}

fn advance_to_next_stable_state(battle: &mut BattleState, context: &mut SearchContext<'_>) {
    let mut safety = 0usize;
    while safety < SAFETY_ACTION_CAP && Instant::now() < context.deadline {
        if matches!(battle.status, BattleStatus::GameOver { .. }) || !is_tactical_state(battle) {
            return;
        }
        let Some(actor) = legal_actions::next_to_act(battle) else {
            return;
        };
        let action = choose_tactical_action(battle, actor, context.deadline, context.root_player);
        battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(battle, actor, action);
        safety += 1;
    }
}

fn choose_tactical_action(
    battle: &BattleState,
    actor: PlayerName,
    deadline: Instant,
    perspective: PlayerName,
) -> BattleAction {
    let legal = legal_actions::compute(battle, actor);
    let actions = legal.all();
    let mut best_action = actions.first().copied().unwrap_or(BattleAction::EndTurn);
    let mut best_score = if actor == perspective { f64::NEG_INFINITY } else { f64::INFINITY };
    let mut context =
        TacticalContext { deadline, perspective, visited: HashSet::new(), memo: HashMap::new() };

    for action in actions.into_iter().take(MAX_TACTICAL_ACTIONS) {
        if Instant::now() >= deadline {
            break;
        }
        let mut next_battle = battle.logical_clone();
        next_battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(&mut next_battle, actor, action);
        let score = tactical_value(&next_battle, TACTICAL_DEPTH, &mut context);
        if actor == perspective {
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

struct TacticalContext {
    deadline: Instant,
    perspective: PlayerName,
    visited: HashSet<u64>,
    memo: HashMap<(u64, u8, PlayerName), f64>,
}

fn tactical_value(battle: &BattleState, depth: u8, context: &mut TacticalContext) -> f64 {
    if Instant::now() >= context.deadline || matches!(battle.status, BattleStatus::GameOver { .. })
    {
        return heuristic_value(battle, context.perspective);
    }
    if depth == 0 || !is_tactical_state(battle) {
        return heuristic_value(battle, context.perspective);
    }
    let state = state_key::key(battle);
    let Some(actor) = legal_actions::next_to_act(battle) else {
        return heuristic_value(battle, context.perspective);
    };
    if !context.visited.insert(state) {
        return heuristic_value(battle, context.perspective) - LOOP_PENALTY;
    }
    if let Some(score) = context.memo.get(&(state, depth, actor)) {
        context.visited.remove(&state);
        return *score;
    }
    let mut best = if actor == context.perspective { f64::NEG_INFINITY } else { f64::INFINITY };
    for action in legal_actions::compute(battle, actor).all().into_iter().take(MAX_TACTICAL_ACTIONS)
    {
        let mut next_battle = battle.logical_clone();
        next_battle.request_context.logging_options.enable_action_legality_check = false;
        apply_battle_action::execute(&mut next_battle, actor, action);
        let score = tactical_value(&next_battle, depth - 1, context);
        if actor == context.perspective {
            best = best.max(score);
        } else {
            best = best.min(score);
        }
    }
    let result = if best.is_finite() { best } else { heuristic_value(battle, context.perspective) };
    context.memo.insert((state, depth, actor), result);
    context.visited.remove(&state);
    result
}

fn heuristic_value(battle: &BattleState, perspective: PlayerName) -> f64 {
    feature_extraction::extract_state_features(battle, perspective)
        .get("state_eval_points_margin")
        .copied()
        .unwrap_or_default();
    state_value_from_features(&feature_extraction::extract_state_features(battle, perspective))
}

fn state_value(battle: &BattleState, perspective: PlayerName, context: &SearchContext<'_>) -> f64 {
    state_value_from_features(&feature_extraction::extract_state_features(battle, perspective))
        + context
            .models
            .value
            .score(&feature_extraction::extract_state_features(battle, perspective))
}

fn state_value_from_features(features: &crate::dataset::FeatureMap) -> f64 {
    features.get("eval_points_margin").copied().unwrap_or_default() * 0.05
        + features.get("eval_front_pressure").copied().unwrap_or_default() * 0.02
        + features.get("eval_blocker_coverage").copied().unwrap_or_default() * 0.02
}

fn fallback_candidate(battle: &BattleState, player: PlayerName) -> ScoredCandidate {
    let action = legal_actions::compute(battle, player)
        .all()
        .first()
        .copied()
        .unwrap_or(BattleAction::EndTurn);
    ScoredCandidate {
        action,
        combined_score: 0.0,
        description: action.battle_action_string(),
        pending_actions: Vec::new(),
        policy_score: 0.0,
        value_score: 0.0,
    }
}

fn prune_candidates(candidates: Vec<RootCandidate>) -> Vec<RootCandidate> {
    if candidates.len() <= 10 {
        return candidates;
    }

    let mut kept = Vec::new();
    let mut rest = Vec::new();
    for (index, candidate) in candidates.into_iter().enumerate() {
        if index < MAX_KEEP_CANDIDATES {
            kept.push(candidate);
        } else {
            rest.push(candidate);
        }
    }
    for candidate in rest.into_iter().take(EXPLORATION_SLOTS) {
        kept.push(candidate);
    }
    kept
}
