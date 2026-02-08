use ability_data::condition::Condition;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::variable_value::VariableValue;
use strings::strings;

use crate::serializer::predicate_serializer;
use crate::variables::parser_bindings::VariableBindings;

/// Serializes a condition to its template text representation.
pub fn serialize_condition(condition: &Condition, bindings: &mut VariableBindings) -> String {
    match condition {
        Condition::AlliesThatShareACharacterType { count } => {
            bindings.insert("a".to_string(), VariableValue::Integer(*count));
            strings::with_allies_sharing_type(0).to_string()
        }
        Condition::CardsDiscardedThisTurn { count: 1, predicate } => {
            strings::if_discarded_this_turn(&*predicate_serializer::serialize_card_predicate(
                predicate, bindings,
            ))
            .to_string()
        }
        Condition::CardsDiscardedThisTurn { predicate, .. } => strings::if_discarded_this_turn(
            &*predicate_serializer::serialize_card_predicate(predicate, bindings),
        )
        .to_string(),
        Condition::CardsDrawnThisTurn { count } => {
            bindings.insert("n".to_string(), VariableValue::Integer(*count));
            strings::if_drawn_count_this_turn(0).to_string()
        }
        Condition::CardsInVoidCount { count } => {
            bindings.insert("n".to_string(), VariableValue::Integer(*count));
            strings::while_void_count(0).to_string()
        }
        Condition::DissolvedThisTurn { .. } => {
            strings::if_character_dissolved_this_turn().to_string()
        }
        Condition::PredicateCount { count: 1, predicate } => {
            if let Predicate::Another(CardPredicate::CharacterType(subtype)) = predicate {
                bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            }
            strings::with_allied_subtype(0).to_string()
        }
        Condition::PredicateCount { count, predicate } => strings::with_predicate_condition(
            &*serialize_predicate_count(*count, predicate, bindings),
        )
        .to_string(),
        Condition::ThisCardIsInYourVoid => strings::if_card_in_your_void().to_string(),
    }
}

fn serialize_predicate_count(
    count: u32,
    predicate: &Predicate,
    bindings: &mut VariableBindings,
) -> String {
    match predicate {
        Predicate::Another(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("a".to_string(), VariableValue::Integer(count));
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            strings::with_count_allied_subtype(0, 0).to_string()
        }
        Predicate::Another(CardPredicate::Character) => {
            bindings.insert("a".to_string(), VariableValue::Integer(count));
            strings::with_count_allies(0).to_string()
        }
        Predicate::Another(card_predicate) => predicate_serializer::serialize_predicate_plural(
            &Predicate::Another(card_predicate.clone()),
            bindings,
        ),
        Predicate::This => predicate_serializer::serialize_predicate_plural(predicate, bindings),
        Predicate::It => predicate_serializer::serialize_predicate_plural(predicate, bindings),
        Predicate::Them => predicate_serializer::serialize_predicate_plural(predicate, bindings),
        Predicate::That => predicate_serializer::serialize_predicate_plural(predicate, bindings),
        Predicate::Enemy(card_predicate) => predicate_serializer::serialize_predicate_plural(
            &Predicate::Enemy(card_predicate.clone()),
            bindings,
        ),
        Predicate::Your(card_predicate) => predicate_serializer::serialize_predicate_plural(
            &Predicate::Your(card_predicate.clone()),
            bindings,
        ),
        Predicate::Any(card_predicate) => predicate_serializer::serialize_predicate_plural(
            &Predicate::Any(card_predicate.clone()),
            bindings,
        ),
        Predicate::AnyOther(card_predicate) => predicate_serializer::serialize_predicate_plural(
            &Predicate::AnyOther(card_predicate.clone()),
            bindings,
        ),
        Predicate::YourVoid(card_predicate) => predicate_serializer::serialize_predicate_plural(
            &Predicate::YourVoid(card_predicate.clone()),
            bindings,
        ),
        Predicate::EnemyVoid(card_predicate) => predicate_serializer::serialize_predicate_plural(
            &Predicate::EnemyVoid(card_predicate.clone()),
            bindings,
        ),
    }
}
