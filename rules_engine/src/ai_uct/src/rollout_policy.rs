use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    LegalActions, StandardLegalActions,
};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;

use crate::position_assignment::{self, CharacterPlacement, PositionAssignment};

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
