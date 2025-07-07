use ability_data::predicate::{CardPredicate, Predicate};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use core_data::card_types::CardType;
use core_data::types::PlayerName;

use crate::battle_card_queries::card_properties;

/// Returns true if the given [Predicate] currently matches for the given card
/// when triggering an event.
///
/// # Arguments
///
/// * `battle` - The current battle state.
/// * `predicate` - The predicate to check.
/// * `trigger_card_id` - The card ID of the card that triggered the event.
/// * `owning_card_controller` - The controller of the card that owns the
///   trigger ability we are checking.
/// * `owning_card_id` - The card ID of the card that owns the trigger ability
///   we are checking.
pub fn trigger_matches(
    battle: &BattleState,
    predicate: &Predicate,
    trigger_card_id: impl CardIdType,
    owning_card_controller: PlayerName,
    owning_card_id: CardId,
) -> bool {
    let trigger_card_id = trigger_card_id.card_id();

    match predicate {
        Predicate::Enemy(card_predicate) => {
            let card_controller = card_properties::controller(battle, trigger_card_id);
            if card_controller != owning_card_controller.opponent() {
                return false;
            }
            matches_card_predicate(battle, card_predicate, trigger_card_id)
        }
        Predicate::Another(card_predicate) => {
            let card_controller = card_properties::controller(battle, trigger_card_id);
            if card_controller != owning_card_controller {
                return false;
            }
            if owning_card_id == trigger_card_id {
                return false;
            }
            matches_card_predicate(battle, card_predicate, trigger_card_id)
        }
        Predicate::Your(card_predicate) => {
            let card_controller = card_properties::controller(battle, trigger_card_id);
            if card_controller != owning_card_controller {
                return false;
            }
            matches_card_predicate(battle, card_predicate, trigger_card_id)
        }
        Predicate::Any(card_predicate) => {
            matches_card_predicate(battle, card_predicate, trigger_card_id)
        }
        Predicate::AnyOther(card_predicate) => {
            if owning_card_id == trigger_card_id {
                return false;
            }
            matches_card_predicate(battle, card_predicate, trigger_card_id)
        }
        _ => todo!("Implement predicate: {:?}", predicate),
    }
}

fn matches_card_predicate(
    battle: &BattleState,
    predicate: &CardPredicate,
    trigger_card_id: CardId,
) -> bool {
    match predicate {
        CardPredicate::Card => true,
        CardPredicate::Character => {
            matches!(card_properties::card_type(battle, trigger_card_id), CardType::Character(_))
        }
        CardPredicate::Event => {
            card_properties::card_type(battle, trigger_card_id) == CardType::Event
        }
        CardPredicate::CharacterType(character_type) => {
            match card_properties::card_type(battle, trigger_card_id) {
                CardType::Character(card_character_type) => card_character_type == *character_type,
                _ => false,
            }
        }
        CardPredicate::NotCharacterType(character_type) => {
            match card_properties::card_type(battle, trigger_card_id) {
                CardType::Character(card_character_type) => card_character_type != *character_type,
                _ => false,
            }
        }
        _ => todo!("Implement card predicate: {:?}", predicate),
    }
}
