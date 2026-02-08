use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use strings::strings;

use crate::serializer::predicate_serializer;

/// Serializes a cost to its template text representation.
pub fn serialize_cost(cost: &Cost) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => match count {
            CollectionExpression::AnyNumberOf => strings::abandon_any_number_of(
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            CollectionExpression::Exactly(1) => {
                strings::abandon_target(predicate_serializer::serialize_predicate(target))
                    .to_string()
            }
            CollectionExpression::Exactly(n) => strings::abandon_count_allies(*n).to_string(),
            _ => strings::abandon_count_allies(0).to_string(),
        },
        Cost::DiscardCards { count, .. } => strings::discard_cards_cost(*count).to_string(),
        Cost::DiscardHand => strings::discard_your_hand_cost().to_string(),
        Cost::Energy(energy) => strings::energy_cost_value(energy.0).to_string(),
        Cost::LoseMaximumEnergy(amount) => strings::lose_max_energy_cost(*amount).to_string(),
        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                strings::banish_another_in_void().to_string()
            } else {
                strings::banish_cards_from_void(*count).to_string()
            }
        }
        Cost::BanishCardsFromEnemyVoid(count) => {
            strings::banish_cards_from_enemy_void(*count).to_string()
        }
        Cost::BanishAllCardsFromYourVoidWithMinCount(min_count) => {
            strings::banish_void_min_count(*min_count).to_string()
        }
        Cost::BanishFromHand(predicate) => {
            strings::banish_from_hand_cost(predicate_serializer::serialize_predicate(predicate))
                .to_string()
        }
        Cost::Choice(costs) => costs
            .iter()
            .map(serialize_cost)
            .collect::<Vec<_>>()
            .join(&strings::cost_or_connector().to_string()),
        Cost::ReturnToHand { target, count } => match count {
            CollectionExpression::Exactly(1) => {
                strings::return_target_to_hand(predicate_serializer::serialize_predicate(target))
                    .to_string()
            }
            CollectionExpression::Exactly(n) => strings::return_count_to_hand(
                *n,
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            CollectionExpression::AllButOne => strings::return_all_but_one_to_hand(
                predicate_serializer::predicate_base_text(target),
            )
            .to_string(),
            CollectionExpression::All => {
                strings::return_all_to_hand(predicate_serializer::serialize_predicate(target))
                    .to_string()
            }
            CollectionExpression::AnyNumberOf => strings::return_any_number_to_hand(
                predicate_serializer::serialize_predicate(target),
            )
            .to_string(),
            CollectionExpression::UpTo(n) => strings::return_up_to_to_hand(
                *n,
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            CollectionExpression::EachOther => strings::return_each_other_to_hand(
                predicate_serializer::serialize_predicate(target),
            )
            .to_string(),
            CollectionExpression::OrMore(n) => strings::return_or_more_to_hand(
                *n,
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
        },
        Cost::SpendOneOrMoreEnergy => strings::pay_one_or_more_energy_cost().to_string(),
        Cost::BanishAllCardsFromYourVoid => strings::banish_your_void_cost().to_string(),
        Cost::CostList(costs) => costs
            .iter()
            .map(serialize_cost)
            .collect::<Vec<_>>()
            .join(&strings::cost_and_connector().to_string()),
    }
}

/// Serializes a cost used as a trigger cost, wrapping energy costs with a
/// "pay" prefix.
pub fn serialize_trigger_cost(cost: &Cost) -> String {
    match cost {
        Cost::Energy(_) => strings::pay_prefix(&*serialize_cost(cost)).to_string(),
        _ => serialize_cost(cost),
    }
}
