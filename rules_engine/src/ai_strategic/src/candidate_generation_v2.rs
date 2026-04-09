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

use crate::evaluation;

const MAX_POSITION_ASSIGNMENTS: usize = 6;
const PASS_PRIORITY_PENALTY: f64 = 8.0;
const END_TURN_PENALTY: f64 = 12.0;

#[derive(Clone)]
pub(crate) enum CandidateExecution {
    Action(BattleAction),
    PositionAssignment(PositionAssignment),
}

#[derive(Clone)]
pub(crate) struct RootCandidate {
    pub action: BattleAction,
    pub description: String,
    pub execution: CandidateExecution,
    pub heuristic_score: f64,
    pub pending_actions: Vec<BattleAction>,
}

pub(crate) fn apply_root_candidate(
    battle: &mut BattleState,
    player: PlayerName,
    candidate: &RootCandidate,
) {
    match &candidate.execution {
        CandidateExecution::Action(action) => {
            apply_battle_action::execute(battle, player, *action);
        }
        CandidateExecution::PositionAssignment(assignment) => {
            rollout_policy::apply_position_assignment(battle, player, assignment);
        }
    }
}

pub(crate) fn generate_root_candidates(
    battle: &BattleState,
    player: PlayerName,
) -> Vec<RootCandidate> {
    let LegalActions::Standard { actions } = legal_actions::compute(battle, player) else {
        return Vec::new();
    };

    let mut candidates = action_candidates(battle, player, &actions);
    if actions.can_begin_positioning {
        candidates.extend(position_candidates(battle, player));
    }
    candidates.sort_by(|left, right| right.heuristic_score.total_cmp(&left.heuristic_score));
    candidates
}

fn action_candidates(
    battle: &BattleState,
    player: PlayerName,
    actions: &StandardLegalActions,
) -> Vec<RootCandidate> {
    standard_actions(actions)
        .into_iter()
        .map(|action| {
            let mut next_battle = battle.logical_clone();
            next_battle.request_context.logging_options.enable_action_legality_check = false;
            apply_battle_action::execute(&mut next_battle, player, action);
            RootCandidate {
                action,
                description: action.battle_action_string(),
                execution: CandidateExecution::Action(action),
                heuristic_score: evaluation::score(&next_battle, player) + action_bias(action),
                pending_actions: Vec::new(),
            }
        })
        .collect()
}

fn position_candidates(battle: &BattleState, player: PlayerName) -> Vec<RootCandidate> {
    let mut assignments = position_assignment::generate(battle, player);
    assignments.sort_by(|left, right| {
        let mut left_battle = battle.logical_clone();
        let mut right_battle = battle.logical_clone();
        rollout_policy::apply_position_assignment(&mut left_battle, player, left);
        rollout_policy::apply_position_assignment(&mut right_battle, player, right);
        evaluation::score(&right_battle, player).total_cmp(&evaluation::score(&left_battle, player))
    });

    assignments
        .into_iter()
        .take(MAX_POSITION_ASSIGNMENTS)
        .map(|assignment| {
            let mut next_battle = battle.logical_clone();
            rollout_policy::apply_position_assignment(&mut next_battle, player, &assignment);
            RootCandidate {
                action: BattleAction::BeginPositioning,
                description: format!(
                    "position:{}",
                    position_assignment::describe(battle, player, &assignment)
                ),
                execution: CandidateExecution::PositionAssignment(assignment.clone()),
                heuristic_score: evaluation::score(&next_battle, player) + 4.0,
                pending_actions: assignment_pending_actions(&assignment),
            }
        })
        .collect()
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
    for &(character_id, position) in &actions.reposition_to_front {
        result.push(BattleAction::MoveCharacterToFrontRank(character_id, position));
    }
    for &(character_id, position) in &actions.reposition_to_back {
        result.push(BattleAction::MoveCharacterToBackRank(character_id, position));
    }
    result.push(match actions.primary {
        PrimaryLegalAction::PassPriority => BattleAction::PassPriority,
        PrimaryLegalAction::EndTurn => BattleAction::EndTurn,
        PrimaryLegalAction::StartNextTurn => BattleAction::StartNextTurn,
    });
    result
}

fn assignment_pending_actions(assignment: &PositionAssignment) -> Vec<BattleAction> {
    let mut pending_actions = Vec::new();
    for &(character_id, placement) in &assignment.placements {
        if let position_assignment::CharacterPlacement::MoveToFrontRank(column) = placement {
            pending_actions.push(BattleAction::SelectCharacterForPositioning(character_id));
            pending_actions.push(BattleAction::MoveCharacterToFrontRank(character_id, column));
        }
    }
    pending_actions.push(BattleAction::EndTurn);
    pending_actions
}

fn action_bias(action: BattleAction) -> f64 {
    match action {
        BattleAction::PassPriority => -PASS_PRIORITY_PENALTY,
        BattleAction::EndTurn => -END_TURN_PENALTY,
        _ => 0.0,
    }
}
