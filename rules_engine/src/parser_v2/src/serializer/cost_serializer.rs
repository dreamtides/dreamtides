use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::variable_value::VariableValue;

use super::predicate_serializer;
use crate::variables::parser_bindings::VariableBindings;
use crate::variables::parser_substitutions;

pub fn serialize_cost(cost: &Cost, bindings: &mut VariableBindings) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => match count {
            CollectionExpression::Exactly(1) => {
                format!("abandon {}", predicate_serializer::serialize_predicate(target, bindings))
            }
            _ => "abandon {count-allies}".to_string(),
        },
        Cost::DiscardCards { target, count } => {
            if *count == 1 {
                format!("discard {}", predicate_serializer::serialize_predicate(target, bindings))
            } else {
                if let Some(var_name) =
                    parser_substitutions::directive_to_integer_variable("discards")
                {
                    bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
                }
                "discard {discards}".to_string()
            }
        }
        Cost::DiscardHand => "discard your hand".to_string(),
        Cost::Energy(energy) => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(energy.0));
            }
            "{e}".to_string()
        }
        Cost::LoseMaximumEnergy(amount) => {
            if let Some(var_name) =
                parser_substitutions::directive_to_integer_variable("maximum-energy")
            {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*amount));
            }
            "lose {maximum-energy}".to_string()
        }
        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                "{Banish} another card in your void".to_string()
            } else {
                if let Some(var_name) = parser_substitutions::directive_to_integer_variable("cards")
                {
                    bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
                }
                "{Banish} {cards} from your void".to_string()
            }
        }
        Cost::BanishCardsFromEnemyVoid(count) => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("cards") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "{Banish} {cards} from the opponent's void".to_string()
        }
        Cost::BanishAllCardsFromYourVoidWithMinCount(min_count) => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("count") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*min_count));
            }
            "{Banish} your void with {count} or more cards".to_string()
        }
        Cost::BanishFromHand(predicate) => {
            format!(
                "{{Banish}} {} from hand",
                predicate_serializer::serialize_predicate(predicate, bindings)
            )
        }
        Cost::Choice(costs) => {
            costs.iter().map(|c| serialize_cost(c, bindings)).collect::<Vec<_>>().join(" or ")
        }
        Cost::ReturnToHand { target, count } => match count {
            CollectionExpression::Exactly(1) => {
                format!(
                    "return {} to hand",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
            CollectionExpression::Exactly(n) => {
                format!(
                    "return {} {} to hand",
                    n,
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::AllButOne => {
                format!(
                    "return all but one {} to hand",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
            CollectionExpression::All => {
                format!(
                    "return all {} to hand",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
            CollectionExpression::AnyNumberOf => {
                format!(
                    "return any number of {} to hand",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
            CollectionExpression::UpTo(n) => {
                format!(
                    "return up to {} {} to hand",
                    n,
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::EachOther => {
                format!(
                    "return each other {} to hand",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
            CollectionExpression::OrMore(n) => {
                format!(
                    "return {} or more {} to hand",
                    n,
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
        },
        Cost::SpendOneOrMoreEnergy => "pay 1 or more {energy-symbol}".to_string(),
        Cost::BanishAllCardsFromYourVoid => "{Banish} your void".to_string(),
        Cost::CostList(costs) => {
            costs.iter().map(|c| serialize_cost(c, bindings)).collect::<Vec<_>>().join(" and ")
        }
    }
}

pub fn serialize_trigger_cost(cost: &Cost, bindings: &mut VariableBindings) -> String {
    match cost {
        Cost::Energy(_) => format!("pay {}", serialize_cost(cost, bindings)),
        _ => serialize_cost(cost, bindings),
    }
}
