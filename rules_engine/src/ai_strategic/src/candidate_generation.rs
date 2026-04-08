use std::collections::HashMap;
use std::time::Instant;

use ai_uct::position_assignment::{self, PositionAssignment};
use ai_uct::rollout_policy;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    LegalActions, PrimaryLegalAction, StandardLegalActions,
};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;

use crate::{evaluation, state_key};

#[derive(Clone)]
pub(crate) enum PlanStep {
    Action(BattleAction),
    Assignment(PositionAssignment),
}

#[derive(Clone)]
pub(crate) struct CandidatePlan {
    pub description: String,
    pub first_action: BattleAction,
    pub heuristic_score: f64,
    pub pending_actions: Vec<BattleAction>,
    pub steps: Vec<PlanStep>,
}

#[derive(Clone)]
struct PartialPlan {
    battle: BattleState,
    description: String,
    heuristic_score: f64,
    pending_actions: Vec<BattleAction>,
    steps: Vec<PlanStep>,
}

pub(crate) fn generate_root_candidates(
    battle: &BattleState,
    player: PlayerName,
    beam_width_root: usize,
    beam_width_rest: usize,
    max_depth: usize,
    deadline: Instant,
) -> Vec<CandidatePlan> {
    let mut beam = vec![PartialPlan {
        battle: battle.logical_clone(),
        description: String::new(),
        heuristic_score: evaluation::score(battle, player),
        pending_actions: Vec::new(),
        steps: Vec::new(),
    }];
    let mut results = Vec::new();

    for depth in 0..max_depth {
        if Instant::now() >= deadline {
            break;
        }

        let mut next = HashMap::new();
        for partial in &beam {
            if !partial.steps.is_empty() {
                results.push(to_candidate(partial));
            }

            if !is_proactive_turn(&partial.battle, player) {
                continue;
            }

            for (step, description, step_pending) in ranked_plan_steps(&partial.battle, player) {
                if Instant::now() >= deadline {
                    break;
                }

                let mut next_battle = partial.battle.logical_clone();
                apply_plan_step(&mut next_battle, player, &step);

                let step_description = if partial.description.is_empty() {
                    description
                } else {
                    format!("{} -> {}", partial.description, description)
                };
                let heuristic_score = evaluation::score(&next_battle, player);
                let mut pending_actions = partial.pending_actions.clone();
                if partial.steps.is_empty() {
                    pending_actions = step_pending;
                }
                let mut steps = partial.steps.clone();
                steps.push(step);
                let candidate = PartialPlan {
                    battle: next_battle,
                    description: step_description,
                    heuristic_score,
                    pending_actions,
                    steps,
                };
                let state_key = state_key::key(&candidate.battle);
                next.entry(state_key)
                    .and_modify(|existing: &mut PartialPlan| {
                        if candidate.heuristic_score > existing.heuristic_score {
                            *existing = candidate.clone();
                        }
                    })
                    .or_insert(candidate);
            }
        }

        if next.is_empty() {
            break;
        }

        let mut next_beam: Vec<_> = next.into_values().collect();
        next_beam.sort_by(|a, b| b.heuristic_score.total_cmp(&a.heuristic_score));
        let beam_width = if depth == 0 { beam_width_root } else { beam_width_rest };
        beam = next_beam.into_iter().take(beam_width).collect();
    }

    for partial in beam {
        if !partial.steps.is_empty() {
            results.push(to_candidate(&partial));
        }
    }

    results.sort_by(|a, b| b.heuristic_score.total_cmp(&a.heuristic_score));
    results.truncate(beam_width_root.max(1));
    results
}

pub(crate) fn apply_candidate_plan(
    battle: &mut BattleState,
    player: PlayerName,
    plan: &CandidatePlan,
) {
    for step in &plan.steps {
        if legal_actions::next_to_act(battle) != Some(player) {
            break;
        }
        if !is_proactive_turn(battle, player) && !matches!(step, PlanStep::Assignment(_)) {
            break;
        }
        apply_plan_step(battle, player, step);
    }
}

pub(crate) fn is_proactive_turn(battle: &BattleState, player: PlayerName) -> bool {
    if legal_actions::next_to_act(battle) != Some(player) {
        return false;
    }

    if battle.cards.has_stack()
        || battle.stack_priority.is_some()
        || battle.turn.positioning_started
        || battle.turn.positioning_character.is_some()
        || !battle.prompts.is_empty()
    {
        return false;
    }

    matches!(legal_actions::compute(battle, player), LegalActions::Standard { .. })
}

fn to_candidate(partial: &PartialPlan) -> CandidatePlan {
    CandidatePlan {
        description: partial.description.clone(),
        first_action: first_action(&partial.steps),
        heuristic_score: partial.heuristic_score,
        pending_actions: partial.pending_actions.clone(),
        steps: partial.steps.clone(),
    }
}

fn first_action(steps: &[PlanStep]) -> BattleAction {
    match steps.first().expect("Candidate plan must contain a first step") {
        PlanStep::Action(action) => *action,
        PlanStep::Assignment(..) => BattleAction::BeginPositioning,
    }
}

fn ranked_plan_steps(
    battle: &BattleState,
    player: PlayerName,
) -> Vec<(PlanStep, String, Vec<BattleAction>)> {
    let LegalActions::Standard { actions } = legal_actions::compute(battle, player) else {
        return Vec::new();
    };
    let mut ranked = Vec::new();

    for action in standard_actions(&actions) {
        let mut next_battle = battle.logical_clone();
        apply_battle_action::execute(&mut next_battle, player, action);
        ranked.push((
            evaluation::score(&next_battle, player),
            PlanStep::Action(action),
            action.battle_action_string(),
            Vec::new(),
        ));
    }

    if actions.can_begin_positioning {
        let mut assignments = position_assignment::generate(battle, player);
        assignments.sort_by(|left, right| {
            let mut left_battle = battle.logical_clone();
            let mut right_battle = battle.logical_clone();
            rollout_policy::apply_position_assignment(&mut left_battle, player, left);
            rollout_policy::apply_position_assignment(&mut right_battle, player, right);
            evaluation::score(&right_battle, player)
                .total_cmp(&evaluation::score(&left_battle, player))
        });

        for assignment in assignments.into_iter().take(6) {
            let mut next_battle = battle.logical_clone();
            rollout_policy::apply_position_assignment(&mut next_battle, player, &assignment);
            let description = position_assignment::describe(battle, player, &assignment);
            ranked.push((
                evaluation::score(&next_battle, player),
                PlanStep::Assignment(assignment.clone()),
                format!("position:{description}"),
                assignment_pending_actions(&assignment),
            ));
        }
    }

    ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
    ranked.into_iter().map(|(_, step, description, pending)| (step, description, pending)).collect()
}

fn apply_plan_step(battle: &mut BattleState, player: PlayerName, step: &PlanStep) {
    match step {
        PlanStep::Action(action) => {
            apply_battle_action::execute(battle, player, *action);
        }
        PlanStep::Assignment(assignment) => {
            rollout_policy::apply_position_assignment(battle, player, assignment);
        }
    }
}

fn standard_actions(actions: &StandardLegalActions) -> Vec<BattleAction> {
    let mut result = Vec::new();
    for card_id in actions.play_card_from_hand.iter() {
        result.push(BattleAction::PlayCardFromHand(card_id));
    }
    for card_id in actions.play_card_from_void.iter() {
        result.push(BattleAction::PlayCardFromVoid(card_id));
    }
    for character_id in actions.activate_abilities_for_character.iter() {
        result.push(BattleAction::ActivateAbilityForCharacter(character_id));
    }
    result.push(match actions.primary {
        PrimaryLegalAction::PassPriority => BattleAction::PassPriority,
        PrimaryLegalAction::EndTurn => BattleAction::EndTurn,
        PrimaryLegalAction::StartNextTurn => BattleAction::StartNextTurn,
    });
    result
}

fn assignment_pending_actions(assignment: &PositionAssignment) -> Vec<BattleAction> {
    let mut actions = Vec::new();
    for (character_id, placement) in &assignment.placements {
        if let position_assignment::CharacterPlacement::MoveToFrontRank(column) = placement {
            actions.push(BattleAction::SelectCharacterForPositioning(*character_id));
            actions.push(BattleAction::MoveCharacterToFrontRank(*character_id, *column));
        }
    }
    actions.push(BattleAction::EndTurn);
    actions
}
