use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::variable_value::VariableValue;

use crate::serializer::predicate_serializer;
use crate::variables::parser_bindings::VariableBindings;

pub fn serialize_cost(cost: &Cost, bindings: &mut VariableBindings) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => match count {
            CollectionExpression::AnyNumberOf => {
                format!(
                    "abandon any number of {}",
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::Exactly(1) => {
                format!("abandon {}", predicate_serializer::serialize_predicate(target, bindings))
            }
            CollectionExpression::Exactly(n) => {
                bindings.insert("a".to_string(), VariableValue::Integer(*n));
                "abandon {count_allies(a)}".to_string()
            }
            _ => "abandon {count_allies(a)}".to_string(),
        },
        Cost::DiscardCards { count, .. } => {
            bindings.insert("d".to_string(), VariableValue::Integer(*count));
            "discard {cards(d)}".to_string()
        }
        Cost::DiscardHand => "discard your hand".to_string(),
        Cost::Energy(energy) => {
            bindings.insert("e".to_string(), VariableValue::Integer(energy.0));
            "{energy(e)}".to_string()
        }
        Cost::LoseMaximumEnergy(amount) => {
            bindings.insert("m".to_string(), VariableValue::Integer(*amount));
            "lose {maximum_energy(m)}".to_string()
        }
        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                "{Banish} another card in your void".to_string()
            } else {
                bindings.insert("c".to_string(), VariableValue::Integer(*count));
                "{Banish} {cards(c)} from your void".to_string()
            }
        }
        Cost::BanishCardsFromEnemyVoid(count) => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            "{Banish} {cards(c)} from the opponent's void".to_string()
        }
        Cost::BanishAllCardsFromYourVoidWithMinCount(min_count) => {
            bindings.insert("n".to_string(), VariableValue::Integer(*min_count));
            "{Banish} your void with {count(n)} or more cards".to_string()
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
                    predicate_serializer::predicate_base_text(target, bindings)
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
        Cost::SpendOneOrMoreEnergy => "pay 1 or more {energy_symbol}".to_string(),
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
