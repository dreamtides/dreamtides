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

use crate::dataset::FeatureMap;
use crate::feature_extraction;

const MAX_ASSIGNMENTS: usize = 4;

#[derive(Clone)]
pub enum CandidateExecution {
    Action(BattleAction),
    PositionAssignment(PositionAssignment),
}

#[derive(Clone)]
pub struct RootCandidate {
    pub action: BattleAction,
    pub action_features: FeatureMap,
    pub description: String,
    pub execution: CandidateExecution,
    pub pending_actions: Vec<BattleAction>,
    pub policy_score: f64,
}

pub fn apply_candidate(battle: &mut BattleState, player: PlayerName, candidate: &RootCandidate) {
    match &candidate.execution {
        CandidateExecution::Action(action) => {
            apply_battle_action::execute(battle, player, *action);
        }
        CandidateExecution::PositionAssignment(assignment) => {
            rollout_policy::apply_position_assignment(battle, player, assignment);
        }
    }
}

pub fn generate_candidates(battle: &BattleState, player: PlayerName) -> Vec<RootCandidate> {
    let LegalActions::Standard { actions } = legal_actions::compute(battle, player) else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for action in standard_actions(&actions) {
        candidates.push(RootCandidate {
            action,
            action_features: feature_extraction::extract_action_features(battle, player, action),
            description: action.battle_action_string(),
            execution: CandidateExecution::Action(action),
            pending_actions: Vec::new(),
            policy_score: 0.0,
        });
    }

    if actions.can_begin_positioning {
        let mut assignments = position_assignment::generate(battle, player);
        assignments.sort_by(|left, right| {
            let mut left_battle = battle.logical_clone();
            let mut right_battle = battle.logical_clone();
            rollout_policy::apply_position_assignment(&mut left_battle, player, left);
            rollout_policy::apply_position_assignment(&mut right_battle, player, right);
            crate::evaluation::score(&right_battle, player)
                .total_cmp(&crate::evaluation::score(&left_battle, player))
        });

        for assignment in assignments.into_iter().take(MAX_ASSIGNMENTS) {
            let mut action_features = feature_extraction::extract_action_features(
                battle,
                player,
                BattleAction::BeginPositioning,
            );
            for (key, value) in
                feature_extraction::extract_assignment_features(battle, player, &assignment)
            {
                action_features.insert(key, value);
            }
            candidates.push(RootCandidate {
                action: BattleAction::BeginPositioning,
                action_features,
                description: format!(
                    "position:{}",
                    position_assignment::describe(battle, player, &assignment)
                ),
                execution: CandidateExecution::PositionAssignment(assignment.clone()),
                pending_actions: assignment_pending_actions(&assignment),
                policy_score: 0.0,
            });
        }
    }

    candidates
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
