use ability_data::predicate::{CardPredicate, Predicate};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::core::effect_source::EffectSource;
use core_data::card_types::CardType;

use crate::battle_card_queries::card_properties;

/// Returns true if the given [Predicate] currently matches for the given card.
pub fn matches(
    battle: &BattleState,
    source: EffectSource,
    predicate: &Predicate,
    card_id: impl CardIdType,
) -> bool {
    let card_id = card_id.card_id();
    let controller = source.controller();

    match predicate {
        Predicate::Enemy(card_predicate) => {
            let card_controller = card_properties::controller(battle, card_id);
            if card_controller != controller.opponent() {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::Another(card_predicate) => {
            let card_controller = card_properties::controller(battle, card_id);
            if card_controller != controller {
                return false;
            }
            if source.card_id() == Some(card_id) {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::Your(card_predicate) => {
            let card_controller = card_properties::controller(battle, card_id);
            if card_controller != controller {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::Any(card_predicate) => {
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::AnyOther(card_predicate) => {
            if source.card_id() == Some(card_id) {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        _ => todo!("Implement predicate: {:?}", predicate),
    }
}

fn matches_card_predicate(
    battle: &BattleState,
    _source: EffectSource,
    predicate: &CardPredicate,
    card_id: CardId,
) -> bool {
    match predicate {
        CardPredicate::Card => true,
        CardPredicate::Character => {
            matches!(card_properties::card_type(battle, card_id), CardType::Character(_))
        }
        CardPredicate::Event => card_properties::card_type(battle, card_id) == CardType::Event,
        CardPredicate::CharacterType(character_type) => {
            match card_properties::card_type(battle, card_id) {
                CardType::Character(card_character_type) => card_character_type == *character_type,
                _ => false,
            }
        }
        CardPredicate::NotCharacterType(character_type) => {
            match card_properties::card_type(battle, card_id) {
                CardType::Character(card_character_type) => card_character_type != *character_type,
                _ => false,
            }
        }
        _ => todo!("Implement card predicate: {:?}", predicate),
    }
}
