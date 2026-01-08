use ability_data::condition::Condition;
use ability_data::predicate::{CardPredicate, Predicate};

use crate::serializer::predicate_serializer;

pub fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::AlliesThatShareACharacterType { .. } => {
            "with {count-allies} that share a character type,".to_string()
        }
        Condition::CardsDiscardedThisTurn { count: 1, predicate } => format!(
            "if you have discarded {} this turn",
            predicate_serializer::serialize_card_predicate(predicate)
        ),
        Condition::CardsDiscardedThisTurn { predicate, .. } => format!(
            "if you have discarded {predicate} this turn",
            predicate = predicate_serializer::serialize_card_predicate(predicate)
        ),
        Condition::CardsDrawnThisTurn { count } => {
            format!("if you have drawn {count} or more cards this turn")
        }
        Condition::CardsInVoidCount { .. } => {
            "while you have {count} or more cards in your void,".to_string()
        }
        Condition::DissolvedThisTurn { .. } => "if a character dissolved this turn".to_string(),
        Condition::PredicateCount { count: 1, .. } => "with an allied {subtype},".to_string(),
        Condition::PredicateCount { count, predicate } => {
            format!(
                "with {count} {predicate},",
                predicate = serialize_predicate_count(*count, predicate)
            )
        }
        Condition::ThisCardIsInYourVoid => "if this card is in your void, ".to_string(),
    }
}

fn serialize_predicate_count(_count: u32, predicate: &Predicate) -> String {
    match predicate {
        Predicate::Another(CardPredicate::CharacterType(_)) => "{count-allied-subtype}".to_string(),
        Predicate::Another(CardPredicate::Character) => "{count-allies}".to_string(),
        _ => {
            unimplemented!("Serialization not yet implemented for this predicate count type")
        }
    }
}
