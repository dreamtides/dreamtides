use ability_data::condition::Condition;
use ability_data::predicate::{CardPredicate, Predicate};
use rlf::Phrase;
use strings::strings;

use crate::serializer::{predicate_serializer, serializer_utils};

/// Serializes a condition to its template text representation.
pub fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::AlliesThatShareACharacterType { count } => {
            strings::with_allies_sharing_type(*count).to_string()
        }
        Condition::CardsDiscardedThisTurn { count: 1, predicate } => {
            strings::if_discarded_this_turn(predicate_serializer::serialize_predicate(
                &Predicate::Any(predicate.clone()),
            ))
            .to_string()
        }
        Condition::CardsDiscardedThisTurn { predicate, .. } => strings::if_discarded_this_turn(
            predicate_serializer::serialize_predicate(&Predicate::Any(predicate.clone())),
        )
        .to_string(),
        Condition::CardsDrawnThisTurn { count } => {
            strings::if_drawn_count_this_turn(*count).to_string()
        }
        Condition::CardsInVoidCount { count } => strings::while_void_count(*count).to_string(),
        Condition::DissolvedThisTurn { .. } => {
            strings::if_character_dissolved_this_turn().to_string()
        }
        Condition::PredicateCount { count: 1, predicate } => {
            if let Predicate::Another(CardPredicate::CharacterType(subtype)) = predicate {
                strings::with_allied_subtype(serializer_utils::subtype_to_phrase(*subtype))
                    .to_string()
            } else {
                strings::with_predicate_condition(serialize_predicate_count(1, predicate))
                    .to_string()
            }
        }
        Condition::PredicateCount { count, predicate } => {
            strings::with_predicate_condition(serialize_predicate_count(*count, predicate))
                .to_string()
        }
        Condition::ThisCardIsInYourVoid => strings::if_card_in_your_void().to_string(),
    }
}

fn serialize_predicate_count(count: u32, predicate: &Predicate) -> Phrase {
    match predicate {
        Predicate::Another(CardPredicate::CharacterType(subtype)) => {
            strings::with_count_allied_subtype(count, serializer_utils::subtype_to_phrase(*subtype))
        }
        Predicate::Another(CardPredicate::Character) => strings::with_count_allies(count),
        Predicate::Another(card_predicate) => {
            predicate_serializer::serialize_predicate(&Predicate::Another(card_predicate.clone()))
        }
        Predicate::This => predicate_serializer::serialize_predicate(predicate),
        Predicate::It => predicate_serializer::serialize_predicate(predicate),
        Predicate::Them => predicate_serializer::serialize_predicate(predicate),
        Predicate::That => predicate_serializer::serialize_predicate(predicate),
        Predicate::Enemy(card_predicate) => {
            predicate_serializer::serialize_predicate(&Predicate::Enemy(card_predicate.clone()))
        }
        Predicate::Your(card_predicate) => {
            predicate_serializer::serialize_predicate(&Predicate::Your(card_predicate.clone()))
        }
        Predicate::Any(card_predicate) => {
            predicate_serializer::serialize_predicate(&Predicate::Any(card_predicate.clone()))
        }
        Predicate::AnyOther(card_predicate) => {
            predicate_serializer::serialize_predicate(&Predicate::AnyOther(card_predicate.clone()))
        }
        Predicate::YourVoid(card_predicate) => {
            predicate_serializer::serialize_predicate(&Predicate::YourVoid(card_predicate.clone()))
        }
        Predicate::EnemyVoid(card_predicate) => {
            predicate_serializer::serialize_predicate(&Predicate::EnemyVoid(card_predicate.clone()))
        }
    }
}
