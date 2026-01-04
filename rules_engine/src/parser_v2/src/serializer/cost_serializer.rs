use ability_data::cost::Cost;
use ability_data::predicate::CardPredicate;

use super::predicate_serializer::{
    serialize_card_predicate, serialize_card_predicate_plural, serialize_predicate,
};

pub fn serialize_cost(cost: &Cost) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => {
            use ability_data::collection_expression::CollectionExpression;
            match count {
                CollectionExpression::Exactly(1) => {
                    format!("abandon {}", serialize_predicate(target))
                }
                _ => "abandon {count-allies}".to_string(),
            }
        }
        Cost::DiscardCards(predicate, count) => {
            if *count == 1 {
                format!("discard {}", serialize_card_predicate(predicate))
            } else {
                match predicate {
                    CardPredicate::Card => "discard {discards}".to_string(),
                    _ => format!(
                        "discard {{discards}} {}",
                        serialize_card_predicate_plural(predicate)
                    ),
                }
            }
        }
        Cost::DiscardHand => "discard your hand".to_string(),
        Cost::Energy(_) => "{e}".to_string(),
        Cost::AbandonACharacterOrDiscardACard => "abandon an ally or discard a card".to_string(),
        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                "{Banish} another card in your void".to_string()
            } else {
                "{Banish} {count} cards in your void".to_string()
            }
        }
        Cost::BanishYourVoidWithMinCount(_) => {
            "{Banish} your void with {count} or more cards".to_string()
        }
        Cost::BanishFromHand(predicate) => {
            format!("{{Banish}} {} from hand", serialize_predicate(predicate))
        }
        _ => unimplemented!("Serialization not yet implemented for this cost type"),
    }
}

pub fn serialize_trigger_cost(cost: &Cost) -> String {
    match cost {
        Cost::Energy(_) => format!("pay {}", serialize_cost(cost)),
        _ => serialize_cost(cost),
    }
}
