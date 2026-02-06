use ability_data::condition::Condition;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::variable_value::VariableValue;

use crate::serializer::predicate_serializer;
use crate::variables::parser_bindings::VariableBindings;
use crate::variables::parser_substitutions;

pub fn serialize_condition(condition: &Condition, bindings: &mut VariableBindings) -> String {
    match condition {
        Condition::AlliesThatShareACharacterType { count } => {
            if let Some(var_name) =
                parser_substitutions::directive_to_integer_variable("count-allies")
            {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "with {count_allies(allies)} that share a character type,".to_string()
        }
        Condition::CardsDiscardedThisTurn { count: 1, predicate } => format!(
            "if you have discarded {} this turn",
            predicate_serializer::serialize_card_predicate(predicate, bindings)
        ),
        Condition::CardsDiscardedThisTurn { predicate, .. } => format!(
            "if you have discarded {predicate} this turn",
            predicate = predicate_serializer::serialize_card_predicate(predicate, bindings)
        ),
        Condition::CardsDrawnThisTurn { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("count") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "if you have drawn {count(count)} or more cards this turn".to_string()
        }
        Condition::CardsInVoidCount { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("count") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "while you have {count(count)} or more cards in your void,".to_string()
        }
        Condition::DissolvedThisTurn { .. } => "if a character dissolved this turn".to_string(),
        Condition::PredicateCount { count: 1, predicate } => {
            if let Predicate::Another(CardPredicate::CharacterType(subtype)) = predicate {
                bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            }
            "with an allied {subtype(subtype)},".to_string()
        }
        Condition::PredicateCount { count, predicate } => {
            format!(
                "with {predicate},",
                predicate = serialize_predicate_count(*count, predicate, bindings)
            )
        }
        Condition::ThisCardIsInYourVoid => "while this card is in your void,".to_string(),
    }
}

fn serialize_predicate_count(
    count: u32,
    predicate: &Predicate,
    bindings: &mut VariableBindings,
) -> String {
    match predicate {
        Predicate::Another(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("allies".to_string(), VariableValue::Integer(count));
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            "{count_allied_subtype(subtype, allies)}".to_string()
        }
        Predicate::Another(CardPredicate::Character) => {
            if let Some(var_name) =
                parser_substitutions::directive_to_integer_variable("count-allies")
            {
                bindings.insert(var_name.to_string(), VariableValue::Integer(count));
            }
            "{count_allies(allies)}".to_string()
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
