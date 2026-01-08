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
            "with {count-allies} that share a character type,".to_string()
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
            format!("if you have drawn {count} or more cards this turn")
        }
        Condition::CardsInVoidCount { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("count") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "while you have {count} or more cards in your void,".to_string()
        }
        Condition::DissolvedThisTurn { .. } => "if a character dissolved this turn".to_string(),
        Condition::PredicateCount { count: 1, .. } => "with an allied {subtype},".to_string(),
        Condition::PredicateCount { count, predicate } => {
            format!(
                "with {count} {predicate},",
                predicate = serialize_predicate_count(*count, predicate, bindings)
            )
        }
        Condition::ThisCardIsInYourVoid => "if this card is in your void, ".to_string(),
    }
}

fn serialize_predicate_count(
    count: u32,
    predicate: &Predicate,
    bindings: &mut VariableBindings,
) -> String {
    match predicate {
        Predicate::Another(CardPredicate::CharacterType(_)) => {
            if let Some(var_name) =
                parser_substitutions::directive_to_integer_variable("count-allied-subtype")
            {
                bindings.insert(var_name.to_string(), VariableValue::Integer(count));
            }
            "{count-allied-subtype}".to_string()
        }
        Predicate::Another(CardPredicate::Character) => {
            if let Some(var_name) =
                parser_substitutions::directive_to_integer_variable("count-allies")
            {
                bindings.insert(var_name.to_string(), VariableValue::Integer(count));
            }
            "{count-allies}".to_string()
        }
        _ => {
            unimplemented!("Serialization not yet implemented for this predicate count type")
        }
    }
}
