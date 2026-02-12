use ability_data::condition::Condition;
use ability_data::predicate::Predicate;
use ability_data::static_ability::{CardTypeContext, StandardStaticAbility, StaticAbility};
use strings::strings;

use crate::serializer::{
    condition_serializer, cost_serializer, effect_serializer, predicate_serializer,
};

/// Serializes a [StaticAbility] to its text representation.
pub fn serialize_static_ability(static_ability: &StaticAbility) -> String {
    match static_ability {
        StaticAbility::StaticAbility(ability) => {
            let base = serialize_standard_static_ability(ability);
            strings::effect_with_period(base).to_string()
        }
        StaticAbility::WithOptions(ability) => {
            let base = serialize_standard_static_ability(&ability.ability);
            if let Some(condition) = &ability.condition {
                let conditioned = if matches!(condition, Condition::ThisCardIsInYourVoid) {
                    strings::if_this_card_in_void_prefix(base).to_string()
                } else if matches!(condition, Condition::CardsInVoidCount { .. })
                    || matches!(condition, Condition::PredicateCount { count: 1, .. })
                {
                    strings::condition_prepended(
                        condition_serializer::serialize_condition(condition),
                        base,
                    )
                    .to_string()
                } else {
                    strings::condition_appended(
                        base,
                        condition_serializer::serialize_condition(condition),
                    )
                    .to_string()
                };
                strings::effect_with_period(conditioned).to_string()
            } else {
                strings::effect_with_period(base).to_string()
            }
        }
    }
}

/// Serializes a [StandardStaticAbility] to its text representation without
/// trailing punctuation.
pub fn serialize_standard_static_ability(ability: &StandardStaticAbility) -> String {
    match ability {
        StandardStaticAbility::YourCardsCostIncrease { matching, increase } => {
            strings::your_cards_cost_increase(
                predicate_serializer::serialize_predicate(&Predicate::Any(matching.clone())),
                increase.0,
            )
            .to_string()
        }
        StandardStaticAbility::YourCardsCostReduction { matching, reduction } => {
            strings::your_cards_cost_reduction(
                predicate_serializer::serialize_predicate(&Predicate::Any(matching.clone())),
                reduction.0,
            )
            .to_string()
        }
        StandardStaticAbility::EnemyCardsCostIncrease { matching, increase } => {
            strings::enemy_cards_cost_increase(
                predicate_serializer::serialize_predicate(&Predicate::Any(matching.clone())),
                increase.0,
            )
            .to_string()
        }
        StandardStaticAbility::SparkBonusOtherCharacters { matching, added_spark } => {
            strings::spark_bonus_other_characters(
                predicate_serializer::serialize_predicate(&Predicate::Any(matching.clone())),
                added_spark.0,
            )
            .to_string()
        }
        StandardStaticAbility::AdditionalCostToPlay(cost) => {
            strings::additional_cost_to_play(cost_serializer::serialize_cost(cost)).to_string()
        }
        StandardStaticAbility::PlayForAlternateCost(alt_cost) => {
            let card_type = match alt_cost.card_type {
                Some(CardTypeContext::Character) => strings::this_character().to_string(),
                Some(CardTypeContext::Event) => strings::this_event().to_string(),
                None => strings::this_card().to_string(),
            };
            if let Some(cost) = &alt_cost.additional_cost {
                let capitalized_cost =
                    strings::capitalized_sentence(cost_serializer::serialize_cost(cost))
                        .to_string();
                if alt_cost.if_you_do.is_some() {
                    strings::play_for_alternate_cost_abandon(
                        capitalized_cost,
                        card_type,
                        alt_cost.energy_cost.0,
                    )
                    .to_string()
                } else {
                    strings::play_for_alternate_cost_with_additional(
                        capitalized_cost,
                        card_type,
                        alt_cost.energy_cost.0,
                    )
                    .to_string()
                }
            } else {
                strings::play_for_alternate_cost_simple(card_type, alt_cost.energy_cost.0)
                    .to_string()
            }
        }
        StandardStaticAbility::CharactersInHandHaveFast => {
            strings::characters_in_hand_have_fast().to_string()
        }
        StandardStaticAbility::DisableEnemyMaterializedAbilities => {
            strings::disable_enemy_materialized_abilities().to_string()
        }
        StandardStaticAbility::HasAllCharacterTypes => {
            strings::has_all_character_types().to_string()
        }
        StandardStaticAbility::MultiplyEnergyGainFromCardEffects { multiplier } => {
            strings::multiply_energy_gain(*multiplier).to_string()
        }
        StandardStaticAbility::MultiplyCardDrawFromCardEffects { multiplier } => {
            strings::multiply_card_draw(*multiplier).to_string()
        }
        StandardStaticAbility::OncePerTurnPlayFromVoid { matching } => {
            strings::once_per_turn_play_from_void(predicate_serializer::serialize_predicate(
                &Predicate::Any(matching.clone()),
            ))
            .to_string()
        }
        StandardStaticAbility::RevealTopCardOfYourDeck => strings::reveal_top_card().to_string(),
        StandardStaticAbility::YouMayLookAtTopCardOfYourDeck => {
            strings::you_may_look_at_top_card().to_string()
        }
        StandardStaticAbility::YouMayPlayFromTopOfDeck { matching } => {
            strings::you_may_play_from_top_of_deck(predicate_serializer::serialize_predicate(
                &Predicate::Any(matching.clone()),
            ))
            .to_string()
        }
        StandardStaticAbility::JudgmentTriggersWhenMaterialized { predicate } => {
            strings::judgment_triggers_when_materialized(predicate_serializer::serialize_predicate(
                predicate,
            ))
            .to_string()
        }
        StandardStaticAbility::SparkEqualToPredicateCount { predicate } => {
            strings::spark_equal_to_predicate_count(predicate_serializer::serialize_predicate(
                predicate,
            ))
            .to_string()
        }
        StandardStaticAbility::PlayOnlyFromVoid => strings::play_only_from_void().to_string(),
        StandardStaticAbility::PlayFromHandOrVoidForCost(play_from_hand_or_void) => {
            strings::play_from_hand_or_void_for_cost(play_from_hand_or_void.energy_cost.0)
                .to_string()
        }
        StandardStaticAbility::CardsInYourVoidHaveReclaim { .. } => {
            strings::cards_in_void_have_reclaim().to_string()
        }
        StandardStaticAbility::CostReductionForEach { reduction, quantity } => {
            strings::cost_reduction_for_each(
                reduction.0,
                effect_serializer::serialize_for_count_expression(quantity),
            )
            .to_string()
        }
        StandardStaticAbility::SparkBonusYourCharacters { matching, added_spark } => {
            strings::spark_bonus_your_characters(
                predicate_serializer::serialize_predicate(&Predicate::Your(matching.clone())),
                added_spark.0,
            )
            .to_string()
        }
        StandardStaticAbility::PlayFromVoid(play_from_void) => {
            if let Some(cost) = &play_from_void.additional_cost {
                let capitalized_cost =
                    strings::capitalized_sentence(cost_serializer::serialize_cost(cost))
                        .to_string();
                if let Some(effect) = &play_from_void.if_you_do {
                    strings::play_from_void_with_effect(
                        capitalized_cost,
                        play_from_void.energy_cost.map_or(0, |e| e.0),
                        effect_serializer::serialize_effect_fragment(effect),
                    )
                    .to_string()
                } else {
                    strings::play_from_void_with_additional_cost(
                        capitalized_cost,
                        play_from_void.energy_cost.map_or(0, |e| e.0),
                    )
                    .to_string()
                }
            } else if let Some(effect) = &play_from_void.if_you_do {
                strings::play_from_void_for_cost_with_effect(
                    play_from_void.energy_cost.map_or(0, |e| e.0),
                    effect_serializer::serialize_effect_fragment(effect),
                )
                .to_string()
            } else {
                strings::play_from_void_for_cost(play_from_void.energy_cost.map_or(0, |e| e.0))
                    .to_string()
            }
        }
    }
}
