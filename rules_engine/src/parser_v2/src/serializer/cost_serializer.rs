use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::variable_value::VariableValue;
use strings::strings;

use crate::serializer::predicate_serializer;
use crate::variables::parser_bindings::VariableBindings;

/// Serializes a cost to its template text representation.
pub fn serialize_cost(cost: &Cost, bindings: &mut VariableBindings) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => match count {
            CollectionExpression::AnyNumberOf => strings::abandon_any_number_of(
                predicate_serializer::serialize_predicate_plural(target, bindings),
            )
            .to_string(),
            CollectionExpression::Exactly(1) => {
                strings::abandon_target(predicate_serializer::serialize_predicate(target, bindings))
                    .to_string()
            }
            CollectionExpression::Exactly(n) => {
                bindings.insert("a".to_string(), VariableValue::Integer(*n));
                strings::abandon_count_allies(0).to_string()
            }
            _ => strings::abandon_count_allies(0).to_string(),
        },
        Cost::DiscardCards { count, .. } => {
            bindings.insert("d".to_string(), VariableValue::Integer(*count));
            strings::discard_cards_cost(0).to_string()
        }
        Cost::DiscardHand => strings::discard_your_hand_cost().to_string(),
        Cost::Energy(energy) => {
            bindings.insert("e".to_string(), VariableValue::Integer(energy.0));
            strings::energy_cost_value(0).to_string()
        }
        Cost::LoseMaximumEnergy(amount) => {
            bindings.insert("m".to_string(), VariableValue::Integer(*amount));
            strings::lose_max_energy_cost(0).to_string()
        }
        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                strings::banish_another_in_void().to_string()
            } else {
                bindings.insert("c".to_string(), VariableValue::Integer(*count));
                strings::banish_cards_from_void(0).to_string()
            }
        }
        Cost::BanishCardsFromEnemyVoid(count) => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            strings::banish_cards_from_enemy_void(0).to_string()
        }
        Cost::BanishAllCardsFromYourVoidWithMinCount(min_count) => {
            bindings.insert("n".to_string(), VariableValue::Integer(*min_count));
            strings::banish_void_min_count(0).to_string()
        }
        Cost::BanishFromHand(predicate) => strings::banish_from_hand_cost(
            predicate_serializer::serialize_predicate(predicate, bindings),
        )
        .to_string(),
        Cost::Choice(costs) => costs
            .iter()
            .map(|c| serialize_cost(c, bindings))
            .collect::<Vec<_>>()
            .join(&strings::cost_or_connector().to_string()),
        Cost::ReturnToHand { target, count } => match count {
            CollectionExpression::Exactly(1) => strings::return_target_to_hand(
                predicate_serializer::serialize_predicate(target, bindings),
            )
            .to_string(),
            CollectionExpression::Exactly(n) => strings::return_count_to_hand(
                *n,
                predicate_serializer::serialize_predicate_plural(target, bindings),
            )
            .to_string(),
            CollectionExpression::AllButOne => strings::return_all_but_one_to_hand(
                predicate_serializer::predicate_base_text(target, bindings),
            )
            .to_string(),
            CollectionExpression::All => strings::return_all_to_hand(
                predicate_serializer::serialize_predicate(target, bindings),
            )
            .to_string(),
            CollectionExpression::AnyNumberOf => strings::return_any_number_to_hand(
                predicate_serializer::serialize_predicate(target, bindings),
            )
            .to_string(),
            CollectionExpression::UpTo(n) => strings::return_up_to_to_hand(
                *n,
                predicate_serializer::serialize_predicate_plural(target, bindings),
            )
            .to_string(),
            CollectionExpression::EachOther => strings::return_each_other_to_hand(
                predicate_serializer::serialize_predicate(target, bindings),
            )
            .to_string(),
            CollectionExpression::OrMore(n) => strings::return_or_more_to_hand(
                *n,
                predicate_serializer::serialize_predicate_plural(target, bindings),
            )
            .to_string(),
        },
        Cost::SpendOneOrMoreEnergy => strings::pay_one_or_more_energy_cost().to_string(),
        Cost::BanishAllCardsFromYourVoid => strings::banish_your_void_cost().to_string(),
        Cost::CostList(costs) => costs
            .iter()
            .map(|c| serialize_cost(c, bindings))
            .collect::<Vec<_>>()
            .join(&strings::cost_and_connector().to_string()),
    }
}

/// Serializes a cost used as a trigger cost, wrapping energy costs with a
/// "pay" prefix.
pub fn serialize_trigger_cost(cost: &Cost, bindings: &mut VariableBindings) -> String {
    match cost {
        Cost::Energy(_) => strings::pay_prefix(&*serialize_cost(cost, bindings)).to_string(),
        _ => serialize_cost(cost, bindings),
    }
}
