use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, CharacterId};
use core_data::card_types::{CardSubtype, CardType};
use core_data::identifiers::CardIdentity;
use core_data::numerics::{Energy, Spark};
use core_data::types::PlayerName;
use quest_state::quest::card_descriptor;
use tabula_ids::test_card;

use crate::battle_card_queries::card;

/// Returns the display name for a card
pub fn display_name(identity: CardIdentity) -> String {
    match card_descriptor::get_base_card_id(identity) {
        test_card::TEST_VANILLA_CHARACTER => "Character".to_string(),
        test_card::TEST_DISSOLVE => "Dissolve".to_string(),
        test_card::TEST_NAMED_DISSOLVE => "Immolate".to_string(),
        test_card::TEST_COUNTERSPELL_UNLESS_PAYS => "Ripple of Defiance".to_string(),
        test_card::TEST_COUNTERSPELL => "Abolish".to_string(),
        test_card::TEST_VARIABLE_ENERGY_DRAW => "Dreamscatter".to_string(),
        test_card::TEST_DRAW_ONE => "Draw One".to_string(),
        test_card::TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER => {
            "Materialize Gain Spark".to_string()
        }
        test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN => "Sundown Surfer".to_string(),
        test_card::TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN => {
            "Play Enemy Gain Spark".to_string()
        }
        test_card::TEST_ACTIVATED_ABILITY_DRAW_CARD => "Activated".to_string(),
        test_card::TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => {
            "Multi Activated".to_string()
        }
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => "Fast Activated".to_string(),
        test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => {
            "Minstrel of Falling Light".to_string()
        }
        test_card::TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER => "Dissolve Character".to_string(),
        test_card::TEST_DUAL_ACTIVATED_ABILITY_CHARACTER => "Dual Abilities".to_string(),
        test_card::TEST_FORESEE_ONE => "Foresee 1".to_string(),
        test_card::TEST_FORESEE_TWO => "Foresee 2".to_string(),
        test_card::TEST_FORESEE_ONE_DRAW_A_CARD => "Foresee 1 Draw 1".to_string(),
        test_card::TEST_DRAW_ONE_RECLAIM => "Draw 1 Reclaim".to_string(),
        test_card::TEST_RETURN_VOID_CARD_TO_HAND => "Return Void Card".to_string(),
        test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND => {
            "Archive of the Forgotten".to_string()
        }
        test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO => "Modal Draw".to_string(),
        test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY => "Modal Draw & Dissolve".to_string(),
        test_card::TEST_RETURN_TO_HAND => "Return to Hand".to_string(),
        test_card::TEST_PREVENT_DISSOLVE_THIS_TURN => "Together Against the Tide".to_string(),
        test_card::TEST_FORESEE_ONE_RECLAIM => "Foresee 1 Reclaim".to_string(),
        test_card::TEST_FORESEE_ONE_DRAW_RECLAIM => "Guiding Light".to_string(),
        test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO => "Break the Sequence".to_string(),
        test_card::TEST_COUNTERSPELL_CHARACTER => "Cragfall".to_string(),
        _ => panic!("Unknown card: {identity:?}"),
    }
}

/// Returns the energy cost of a card, or 0 if it has no energy cost.
///
/// Cards may not have an energy cost due to their card type (e.g. dreamwell
/// cards) or may not have a cost due to their ability (e.g. modal cards).
pub fn converted_energy_cost(battle: &BattleState, card_id: impl CardIdType) -> Energy {
    base_energy_cost_for_id(battle, card_id).unwrap_or_default()
}

/// Returns the base energy cost of a card as in [base_energy_cost].
pub fn base_energy_cost_for_id(battle: &BattleState, card_id: impl CardIdType) -> Option<Energy> {
    card::get(battle, card_id).base_energy_cost
}

/// Returns the base energy cost of a card specified in the card definition, or
/// None if it has no energy cost (e.g. modal cards).
pub fn base_energy_cost(identity: CardIdentity) -> Option<Energy> {
    match card_descriptor::get_base_card_id(identity) {
        test_card::TEST_VANILLA_CHARACTER => Some(Energy(2)),
        test_card::TEST_DISSOLVE => Some(Energy(2)),
        test_card::TEST_NAMED_DISSOLVE => Some(Energy(2)),
        test_card::TEST_COUNTERSPELL_UNLESS_PAYS => Some(Energy(1)),
        test_card::TEST_COUNTERSPELL => Some(Energy(2)),
        test_card::TEST_COUNTERSPELL_CHARACTER => Some(Energy(2)),
        test_card::TEST_VARIABLE_ENERGY_DRAW => Some(Energy(2)),
        test_card::TEST_DRAW_ONE => Some(Energy(0)),
        test_card::TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER => Some(Energy(0)),
        test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN => Some(Energy(2)),
        test_card::TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN => Some(Energy(2)),
        test_card::TEST_ACTIVATED_ABILITY_DRAW_CARD => Some(Energy(2)),
        test_card::TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => Some(Energy(0)),
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => Some(Energy(0)),
        test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => Some(Energy(2)),
        test_card::TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER => Some(Energy(3)),
        test_card::TEST_DUAL_ACTIVATED_ABILITY_CHARACTER => Some(Energy(4)),
        test_card::TEST_FORESEE_ONE => Some(Energy(1)),
        test_card::TEST_FORESEE_TWO => Some(Energy(1)),
        test_card::TEST_FORESEE_ONE_DRAW_A_CARD => Some(Energy(1)),
        test_card::TEST_DRAW_ONE_RECLAIM => Some(Energy(2)),
        test_card::TEST_FORESEE_ONE_RECLAIM => Some(Energy(1)),
        test_card::TEST_FORESEE_ONE_DRAW_RECLAIM => Some(Energy(1)),
        test_card::TEST_RETURN_VOID_CARD_TO_HAND => Some(Energy(1)),
        test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND => Some(Energy(4)),
        test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO => None,
        test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY => None,
        test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO => None,
        test_card::TEST_RETURN_TO_HAND => Some(Energy(1)),
        test_card::TEST_PREVENT_DISSOLVE_THIS_TURN => Some(Energy(1)),
        _ => panic!("Unknown card: {identity:?}"),
    }
}

/// Returns the player who currently controls a given card.
pub fn controller(battle: &BattleState, card_id: impl CardIdType) -> PlayerName {
    card::get(battle, card_id).owner
}

pub fn spark(battle: &BattleState, controller: PlayerName, id: CharacterId) -> Option<Spark> {
    battle.cards.spark(controller, id)
}

pub fn base_spark_for_id(battle: &BattleState, card_id: impl CardIdType) -> Option<Spark> {
    card::get(battle, card_id).base_spark
}

pub fn base_spark(identity: CardIdentity) -> Option<Spark> {
    match card_descriptor::get_base_card_id(identity) {
        test_card::TEST_VANILLA_CHARACTER => Some(Spark(5)),
        test_card::TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER => Some(Spark(5)),
        test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN => Some(Spark(1)),
        test_card::TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN => Some(Spark(5)),
        test_card::TEST_ACTIVATED_ABILITY_DRAW_CARD => Some(Spark(3)),
        test_card::TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => Some(Spark(3)),
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => Some(Spark(3)),
        test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => Some(Spark(2)),
        test_card::TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER => Some(Spark(4)),
        test_card::TEST_DUAL_ACTIVATED_ABILITY_CHARACTER => Some(Spark(3)),
        _ => None,
    }
}

pub fn card_type(battle: &BattleState, card_id: impl CardIdType) -> CardType {
    card::get(battle, card_id).card_type
}

pub fn card_subtype(_battle: &BattleState, _card_id: impl CardIdType) -> Option<CardSubtype> {
    Some(CardSubtype::Musician)
}

pub fn card_type_by_name(identity: CardIdentity) -> CardType {
    match card_descriptor::get_base_card_id(identity) {
        test_card::TEST_VANILLA_CHARACTER => CardType::Character,
        test_card::TEST_DISSOLVE => CardType::Event,
        test_card::TEST_NAMED_DISSOLVE => CardType::Event,
        test_card::TEST_COUNTERSPELL_UNLESS_PAYS => CardType::Event,
        test_card::TEST_COUNTERSPELL => CardType::Event,
        test_card::TEST_VARIABLE_ENERGY_DRAW => CardType::Event,
        test_card::TEST_DRAW_ONE => CardType::Event,
        test_card::TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER => {
            CardType::Character
        }
        test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN => CardType::Character,
        test_card::TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN => CardType::Character,
        test_card::TEST_ACTIVATED_ABILITY_DRAW_CARD => CardType::Character,
        test_card::TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => CardType::Character,
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => CardType::Character,
        test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => CardType::Character,
        test_card::TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER => CardType::Character,
        test_card::TEST_DUAL_ACTIVATED_ABILITY_CHARACTER => CardType::Character,
        test_card::TEST_FORESEE_ONE => CardType::Event,
        test_card::TEST_FORESEE_TWO => CardType::Event,
        test_card::TEST_FORESEE_ONE_DRAW_A_CARD => CardType::Event,
        test_card::TEST_DRAW_ONE_RECLAIM => CardType::Event,
        test_card::TEST_RETURN_VOID_CARD_TO_HAND => CardType::Event,
        test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND => CardType::Event,
        test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO => CardType::Event,
        test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY => CardType::Event,
        test_card::TEST_RETURN_TO_HAND => CardType::Event,
        test_card::TEST_PREVENT_DISSOLVE_THIS_TURN => CardType::Event,
        test_card::TEST_COUNTERSPELL_CHARACTER => CardType::Event,
        test_card::TEST_FORESEE_ONE_RECLAIM => CardType::Event,
        test_card::TEST_FORESEE_ONE_DRAW_RECLAIM => CardType::Event,
        test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO => CardType::Event,
        _ => panic!("Unknown card: {identity:?}"),
    }
}

pub fn is_fast(battle: &BattleState, card_id: impl CardIdType) -> bool {
    card::get(battle, card_id).is_fast
}

pub fn is_fast_by_name(identity: CardIdentity) -> bool {
    match card_descriptor::get_base_card_id(identity) {
        test_card::TEST_VANILLA_CHARACTER => false,
        test_card::TEST_DISSOLVE => true,
        test_card::TEST_NAMED_DISSOLVE => true,
        test_card::TEST_COUNTERSPELL_UNLESS_PAYS => true,
        test_card::TEST_COUNTERSPELL => true,
        test_card::TEST_VARIABLE_ENERGY_DRAW => true,
        test_card::TEST_DRAW_ONE => true,
        test_card::TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER => false,
        test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN => true,
        test_card::TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN => true,
        test_card::TEST_ACTIVATED_ABILITY_DRAW_CARD => false,
        test_card::TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => false,
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => false,
        test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => false,
        test_card::TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER => false,
        test_card::TEST_DUAL_ACTIVATED_ABILITY_CHARACTER => false,
        test_card::TEST_FORESEE_ONE => true,
        test_card::TEST_FORESEE_TWO => true,
        test_card::TEST_FORESEE_ONE_DRAW_A_CARD => true,
        test_card::TEST_DRAW_ONE_RECLAIM => true,
        test_card::TEST_RETURN_VOID_CARD_TO_HAND => true,
        test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND => false,
        test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO => false,
        test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY => true,
        test_card::TEST_RETURN_TO_HAND => true,
        test_card::TEST_PREVENT_DISSOLVE_THIS_TURN => true,
        test_card::TEST_COUNTERSPELL_CHARACTER => true,
        test_card::TEST_FORESEE_ONE_RECLAIM => true,
        test_card::TEST_FORESEE_ONE_DRAW_RECLAIM => true,
        test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO => true,
        _ => panic!("Unknown card: {identity:?}"),
    }
}
