use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;

use super::predicate_serializer::serialize_predicate;

pub fn serialize_cost(cost: &Cost) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => match count {
            CollectionExpression::Exactly(1) => {
                format!("abandon {}", serialize_predicate(target))
            }
            _ => "abandon {count-allies}".to_string(),
        },
        Cost::DiscardCards { target, count } => {
            if *count == 1 {
                format!("discard {}", serialize_predicate(target))
            } else {
                "discard {discards}".to_string()
            }
        }
        Cost::DiscardHand => "discard your hand".to_string(),
        Cost::Energy(_) => "{e}".to_string(),
        Cost::LoseMaximumEnergy(_) => "lose {maximum-energy}".to_string(),
        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                "{Banish} another card in your void".to_string()
            } else {
                "{Banish} {cards} from your void".to_string()
            }
        }
        Cost::BanishCardsFromEnemyVoid(_) => {
            "{Banish} {cards} from the opponent's void".to_string()
        }
        Cost::BanishAllCardsFromYourVoidWithMinCount(_) => {
            "{Banish} your void with {count} or more cards".to_string()
        }
        Cost::BanishFromHand(predicate) => {
            format!("{{Banish}} {} from hand", serialize_predicate(predicate))
        }
        Cost::Choice(costs) => costs.iter().map(serialize_cost).collect::<Vec<_>>().join(" or "),
        Cost::ReturnToHand { target, count } => {
            match count {
                CollectionExpression::Exactly(1) => {
                    format!("return {} to hand", serialize_predicate(target))
                }
                CollectionExpression::AllButOne => {
                    format!("return all but one {} to hand", serialize_predicate(target))
                }
                CollectionExpression::All => {
                    format!("return all {} to hand", serialize_predicate(target))
                }
                CollectionExpression::AnyNumberOf => {
                    format!("return any number of {} to hand", serialize_predicate(target))
                }
                _ => unimplemented!("Serialization not yet implemented for this collection expression in return to hand cost"),
            }
        }
        Cost::SpendOneOrMoreEnergy => "pay 1 or more {energy-symbol}".to_string(),
        _ => unimplemented!("Serialization not yet implemented for this cost type"),
    }
}

pub fn serialize_trigger_cost(cost: &Cost) -> String {
    match cost {
        Cost::Energy(_) => format!("pay {}", serialize_cost(cost)),
        _ => serialize_cost(cost),
    }
}
