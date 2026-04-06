use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_player::battle_player_state::PlayerType;
use battle_state::prompt_types::prompt_data::PromptType;
use bit_set::BitSet;
use core_data::types::PlayerName;

use crate::legal_action_queries::can_play_cards::{self, FastOnly};
use crate::legal_action_queries::legal_actions_data::{
    LegalActions, PrimaryLegalAction, RepositionActions, StandardLegalActions,
};
use crate::legal_action_queries::{can_activate_abilities, legal_modal_effect_choices};

pub fn compute(battle: &BattleState, player: PlayerName) -> LegalActions {
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return LegalActions::NoActionsGameOver;
    }

    // If there's an active prompt, the only legal actions are those
    // corresponding to the prompt
    if let Some(prompt_data) = battle.prompts.front() {
        if prompt_data.player != player {
            return LegalActions::NoActionsOpponentPrompt;
        }

        return match &prompt_data.prompt_type {
            PromptType::ChooseCharacter { valid, .. } => {
                LegalActions::SelectCharacterPrompt { valid: valid.clone() }
            }
            PromptType::ChooseStackCard { valid, .. } => {
                LegalActions::SelectStackCardPrompt { valid: valid.clone() }
            }
            PromptType::ChooseVoidCard(prompt) => LegalActions::SelectVoidCardPrompt {
                valid: prompt.valid.clone(),
                current: prompt.selected.clone(),
                maximum_selection: prompt.maximum_selection as usize,
            },
            PromptType::ChooseHandCards(prompt) => LegalActions::SelectHandCardPrompt {
                valid: prompt.valid.clone(),
                current: prompt.selected.clone(),
                target_count: prompt.maximum_selection as usize,
            },
            PromptType::Choose { choices } => {
                LegalActions::SelectPromptChoicePrompt { choice_count: choices.len() }
            }
            PromptType::ChooseEnergyValue { minimum, maximum } => {
                LegalActions::SelectEnergyValuePrompt { minimum: *minimum, maximum: *maximum }
            }
            PromptType::ModalEffect(prompt) => {
                let source = prompt_data.source;
                let mut valid_choices = BitSet::<usize>::default();
                for (i, choice) in prompt.choices.iter().enumerate() {
                    if legal_modal_effect_choices::is_legal_choice(battle, source, player, choice) {
                        valid_choices.insert(i);
                    }
                }
                LegalActions::ModalEffectPrompt { valid_choices }
            }
            PromptType::ChooseActivatedAbility { abilities, .. } => {
                LegalActions::SelectActivatedAbilityPrompt { choice_count: abilities.len() }
            }
            PromptType::SelectDeckCardOrder { prompt } => {
                LegalActions::SelectDeckCardOrder { current: prompt.clone() }
            }
        };
    }

    if let Some(priority) = battle.stack_priority {
        if priority == player {
            LegalActions::Standard {
                actions: standard_legal_actions(
                    battle,
                    player,
                    PrimaryLegalAction::PassPriority,
                    FastOnly::Yes,
                ),
            }
        } else {
            LegalActions::NoActionsOpponentPriority
        }
    } else if battle.turn.active_player == player && battle.phase == BattleTurnPhase::Main {
        let use_phased = battle.action_history.is_none()
            || matches!(battle.players.player(player).player_type, PlayerType::Agent(_));
        if use_phased {
            phased_main_phase_actions(battle, player)
        } else {
            LegalActions::Standard {
                actions: standard_legal_actions(
                    battle,
                    player,
                    PrimaryLegalAction::EndTurn,
                    FastOnly::No,
                ),
            }
        }
    } else if battle.turn.active_player != player && battle.phase == BattleTurnPhase::Ending {
        LegalActions::Standard {
            actions: standard_legal_actions(
                battle,
                player,
                PrimaryLegalAction::StartNextTurn,
                FastOnly::Yes,
            ),
        }
    } else {
        LegalActions::NoActionsInCurrentPhase
    }
}

/// Returns the player who is next to act.
///
/// Returns None if the game is over.
pub fn next_to_act(battle: &BattleState) -> Option<PlayerName> {
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return None;
    }

    if let Some(prompt_data) = &battle.prompts.front() {
        return Some(prompt_data.player);
    }

    if let Some(priority) = battle.stack_priority {
        Some(priority)
    } else if battle.phase != BattleTurnPhase::Ending {
        Some(battle.turn.active_player)
    } else {
        Some(battle.turn.active_player.opponent())
    }
}

fn standard_legal_actions(
    battle: &BattleState,
    player: PlayerName,
    primary: PrimaryLegalAction,
    fast_only: FastOnly,
) -> StandardLegalActions {
    let (reposition_to_front, reposition_to_back) = if matches!(fast_only, FastOnly::No) {
        reposition_actions(battle, player)
    } else {
        (vec![], vec![])
    };

    StandardLegalActions {
        primary,
        play_card_from_hand: can_play_cards::from_hand(battle, player, fast_only),
        play_card_from_void: can_play_cards::from_void(battle, player, fast_only),
        activate_abilities_for_character: can_activate_abilities::for_player(
            battle, player, fast_only,
        ),
        reposition_to_front,
        reposition_to_back,
        can_begin_positioning: false,
    }
}

/// Full reposition actions for the human player UI.
fn reposition_actions(
    battle: &BattleState,
    player: PlayerName,
) -> (RepositionActions, RepositionActions) {
    let bf = battle.cards.battlefield(player);
    let current_turn = battle.turn.turn_id.0;
    let mut to_front = Vec::new();
    let mut to_back = Vec::new();

    for character_id in bf.back.iter().flatten() {
        let has_summoning_sickness = battle
            .cards
            .battlefield_state(player)
            .get(character_id)
            .is_some_and(|state| state.played_turn == current_turn);
        if !has_summoning_sickness {
            for position in 0..8u8 {
                to_front.push((*character_id, position));
            }
        }

        for position in 0..8u8 {
            if bf.back[position as usize] != Some(*character_id) {
                to_back.push((*character_id, position));
            }
        }
    }

    for character_id in bf.front.iter().flatten() {
        for position in 0..8u8 {
            to_back.push((*character_id, position));
        }

        for position in 0..8u8 {
            if bf.front[position as usize] != Some(*character_id) {
                to_front.push((*character_id, position));
            }
        }
    }

    (to_front, to_back)
}

/// Routes the AI's main phase through a phased state machine: card play
/// first, then multi-step positioning.
fn phased_main_phase_actions(battle: &BattleState, player: PlayerName) -> LegalActions {
    if battle.turn.positioning_character.is_some() {
        return assign_column_actions(battle, player);
    }
    if battle.turn.positioning_started {
        return select_positioning_character_actions(battle, player);
    }
    card_play_phase_actions(battle, player)
}

/// Phase 1: card play. Cards, abilities, BeginPositioning, EndTurn.
fn card_play_phase_actions(battle: &BattleState, player: PlayerName) -> LegalActions {
    let has_eligible = eligible_back_rank_characters(battle, player).next().is_some();
    LegalActions::Standard {
        actions: StandardLegalActions {
            primary: PrimaryLegalAction::EndTurn,
            play_card_from_hand: can_play_cards::from_hand(battle, player, FastOnly::No),
            play_card_from_void: can_play_cards::from_void(battle, player, FastOnly::No),
            activate_abilities_for_character: can_activate_abilities::for_player(
                battle,
                player,
                FastOnly::No,
            ),
            reposition_to_front: vec![],
            reposition_to_back: vec![],
            can_begin_positioning: has_eligible,
        },
    }
}

/// Step A: select which back-rank character to move forward.
fn select_positioning_character_actions(battle: &BattleState, player: PlayerName) -> LegalActions {
    LegalActions::SelectPositioningCharacter {
        eligible: eligible_back_rank_characters(battle, player).collect(),
    }
}

/// Step B: assign the selected character to a column.
fn assign_column_actions(battle: &BattleState, player: PlayerName) -> LegalActions {
    let character = battle.turn.positioning_character.expect("No positioning character set");
    let opponent_front = &battle.cards.battlefield(player.opponent()).front;
    let own_front = &battle.cards.battlefield(player).front;

    let block_targets: Vec<u8> = opponent_front
        .iter()
        .enumerate()
        .filter_map(|(col, slot)| slot.map(|_| col as u8))
        .collect();

    let attack_column = opponent_front
        .iter()
        .zip(own_front.iter())
        .position(|(opp, own)| opp.is_none() && own.is_none())
        .map(|col| col as u8);

    LegalActions::AssignColumn { character, block_targets, attack_column }
}

/// Returns eligible back-rank characters: no summoning sickness, not
/// already moved this turn.
fn eligible_back_rank_characters(
    battle: &BattleState,
    player: PlayerName,
) -> impl Iterator<Item = CharacterId> + '_ {
    let bf = battle.cards.battlefield(player);
    let current_turn = battle.turn.turn_id.0;
    bf.back.iter().flatten().copied().filter(move |character_id| {
        let has_summoning_sickness = battle
            .cards
            .battlefield_state(player)
            .get(character_id)
            .is_some_and(|state| state.played_turn == current_turn);
        !has_summoning_sickness && !battle.turn.moved_this_turn.contains(character_id)
    })
}
