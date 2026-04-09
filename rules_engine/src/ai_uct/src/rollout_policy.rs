use std::cmp::Ordering;

use battle_mutations::actions::apply_battle_action;
use battle_queries::battle_card_queries::card_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    LegalActions, StandardLegalActions,
};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;

use crate::position_assignment::{self, CharacterPlacement, PositionAssignment};

pub struct HybridRolloutChoice {
    pub action: BattleAction,
    pub assignment: Option<PositionAssignment>,
    pub pass_suppressed: bool,
}

/// Atomically applies a position assignment to the battle state via the
/// standard action sequence.
pub fn apply_position_assignment(
    battle: &mut BattleState,
    player: PlayerName,
    assignment: &PositionAssignment,
) {
    apply_battle_action::execute(battle, player, BattleAction::BeginPositioning);
    for &(char_id, placement) in &assignment.placements {
        if let CharacterPlacement::MoveToFrontRank(col) = placement {
            apply_battle_action::execute(
                battle,
                player,
                BattleAction::SelectCharacterForPositioning(char_id),
            );
            apply_battle_action::execute(
                battle,
                player,
                BattleAction::MoveCharacterToFrontRank(char_id, col),
            );
        }
    }
    apply_battle_action::execute(battle, player, BattleAction::EndTurn);
}

/// Plays out a complete turn using greedy heuristics for card play and
/// positioning.
pub fn play_greedy_turn(battle: &mut BattleState, player: PlayerName) {
    greedy_card_play(battle, player);

    let legal = legal_actions::compute(battle, player);
    if let LegalActions::Standard { actions } = &legal
        && actions.can_begin_positioning
        && let Some(assignment) = position_assignment::best_assignment(battle, player)
    {
        apply_position_assignment(battle, player, &assignment);
        return;
    }

    apply_battle_action::execute(battle, player, BattleAction::EndTurn);
}

/// Plays a complete turn for the given player using randomized logic with
/// atomic position assignments.
///
/// Card play uses random action selection. When positioning is available,
/// randomly selects one assignment from the generated candidates. Biases
/// toward positioning when `EndTurn` would otherwise be chosen.
pub fn play_random_turn(battle: &mut BattleState, player: PlayerName) {
    let mut safety = 0;
    loop {
        if safety > 500 {
            return;
        }
        safety += 1;

        let Some(acting_player) = legal_actions::next_to_act(battle) else {
            return;
        };

        if acting_player != player {
            return;
        }

        let legal = legal_actions::compute(battle, player);
        match &legal {
            LegalActions::Standard { actions } => {
                let action = legal.random_action();
                let chose_end_turn = action == Some(BattleAction::EndTurn)
                    || action == Some(BattleAction::StartNextTurn);
                if chose_end_turn && actions.can_begin_positioning {
                    let candidates = position_assignment::generate(battle, player);
                    if !candidates.is_empty() {
                        let index = fastrand::usize(..candidates.len());
                        apply_position_assignment(battle, player, &candidates[index]);
                        return;
                    }
                }

                if actions.can_begin_positioning && action == Some(BattleAction::BeginPositioning) {
                    let candidates = position_assignment::generate(battle, player);
                    if !candidates.is_empty() {
                        let index = fastrand::usize(..candidates.len());
                        apply_position_assignment(battle, player, &candidates[index]);
                        return;
                    }
                }

                match action {
                    Some(BattleAction::EndTurn | BattleAction::StartNextTurn) => return,
                    Some(a) => {
                        apply_battle_action::execute(battle, player, a);
                    }
                    None => return,
                }
            }
            _ => {
                let Some(action) = legal.random_action() else {
                    return;
                };
                apply_battle_action::execute(battle, player, action);
            }
        }
    }
}

pub fn pick_hybrid_rollout_action(
    battle: &BattleState,
    player: PlayerName,
    actions: &StandardLegalActions,
) -> Option<HybridRolloutChoice> {
    let legal = LegalActions::Standard { actions: actions.clone() };
    let all_actions = legal.all();
    if all_actions.is_empty() {
        return None;
    }

    let has_proactive_action = all_actions.iter().copied().any(|action| {
        !matches!(
            action,
            BattleAction::EndTurn | BattleAction::PassPriority | BattleAction::StartNextTurn
        )
    });
    let best_assignment = if actions.can_begin_positioning {
        position_assignment::best_assignment(battle, player)
    } else {
        None
    };

    let mut scored: Vec<_> = all_actions
        .iter()
        .copied()
        .map(|action| (action, score_standard_action(battle, player, actions, action)))
        .collect();
    scored.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(Ordering::Equal));
    let (action, _) = scored.first().copied()?;
    let pass_suppressed = has_proactive_action
        && matches!(
            action,
            BattleAction::EndTurn | BattleAction::PassPriority | BattleAction::StartNextTurn
        );

    if action == BattleAction::BeginPositioning {
        return Some(HybridRolloutChoice { action, assignment: best_assignment, pass_suppressed });
    }

    Some(HybridRolloutChoice { action, assignment: None, pass_suppressed })
}

pub fn score_standard_action(
    battle: &BattleState,
    player: PlayerName,
    actions: &StandardLegalActions,
    action: BattleAction,
) -> f64 {
    match action {
        BattleAction::PlayCardFromHand(card_id) => {
            let spark =
                f64::from(card_properties::base_spark(battle, card_id).unwrap_or_default().0);
            let cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
            let fast_bonus = if card_properties::is_fast(battle, card_id) { 12.0 } else { 0.0 };
            55.0 + spark * 18.0 + cost * 8.0 + fast_bonus
        }
        BattleAction::PlayCardFromVoid(card_id) => {
            let spark =
                f64::from(card_properties::base_spark(battle, card_id).unwrap_or_default().0);
            let cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
            let fast_bonus = if card_properties::is_fast(battle, card_id) { 10.0 } else { 0.0 };
            48.0 + spark * 16.0 + cost * 7.0 + fast_bonus
        }
        BattleAction::ActivateAbilityForCharacter(character_id) => {
            let spark = f64::from(
                card_properties::spark(battle, player, character_id).unwrap_or_default().0,
            );
            28.0 + spark * 10.0
        }
        BattleAction::BeginPositioning => {
            let assignment_score = position_assignment::best_assignment(battle, player)
                .map(|assignment| position_assignment::describe(battle, player, &assignment))
                .map_or(0.0, |description| assignment_description_score(&description));
            36.0 + assignment_score
        }
        BattleAction::PassPriority => {
            if has_non_pass_action(actions) {
                -65.0
            } else {
                -5.0
            }
        }
        BattleAction::EndTurn | BattleAction::StartNextTurn => {
            if has_non_pass_action(actions) {
                -80.0
            } else {
                -10.0
            }
        }
        _ => 0.0,
    }
}

fn greedy_card_play(battle: &mut BattleState, player: PlayerName) {
    loop {
        let legal = legal_actions::compute(battle, player);
        let LegalActions::Standard { actions } = &legal else {
            return;
        };

        if let Some(action) = pick_best_card_action(actions) {
            apply_battle_action::execute(battle, player, action);
            resolve_until_standard(battle, player);
        } else {
            return;
        }
    }
}

fn pick_best_card_action(actions: &StandardLegalActions) -> Option<BattleAction> {
    if let Some(card_id) = actions.play_card_from_hand.iter().next() {
        return Some(BattleAction::PlayCardFromHand(card_id));
    }
    if let Some(card_id) = actions.play_card_from_void.iter().next() {
        return Some(BattleAction::PlayCardFromVoid(card_id));
    }
    if let Some(char_id) = actions.activate_abilities_for_character.iter().next() {
        return Some(BattleAction::ActivateAbilityForCharacter(char_id));
    }
    None
}

fn assignment_description_score(description: &str) -> f64 {
    if description == "hold-all" {
        4.0
    } else {
        let attack_count = description.matches("attack-").count() as f64;
        let block_count = description.matches("block-").count() as f64;
        let chump_count = description.matches("chump-").count() as f64;
        attack_count * 12.0 + block_count * 18.0 + chump_count * 10.0
    }
}

fn has_non_pass_action(actions: &StandardLegalActions) -> bool {
    !actions.play_card_from_hand.is_empty()
        || !actions.play_card_from_void.is_empty()
        || !actions.activate_abilities_for_character.is_empty()
        || actions.can_begin_positioning
}

fn resolve_until_standard(battle: &mut BattleState, player: PlayerName) {
    let mut safety = 0;
    loop {
        if safety > 500 {
            return;
        }
        safety += 1;

        let Some(acting_player) = legal_actions::next_to_act(battle) else {
            return;
        };

        let legal = legal_actions::compute(battle, acting_player);
        if acting_player == player && matches!(legal, LegalActions::Standard { .. }) {
            return;
        }

        let Some(action) = legal.random_action() else {
            return;
        };
        apply_battle_action::execute(battle, acting_player, action);
    }
}
