use ability_data::condition::Condition;
use ability_data::static_ability::{CardTypeContext, StandardStaticAbility, StaticAbility};
use strings::strings;

use crate::serializer::{
    condition_serializer, cost_serializer, effect_serializer, predicate_serializer,
    serializer_utils,
};

/// Serializes a [StaticAbility] to its text representation.
pub fn serialize_static_ability(static_ability: &StaticAbility) -> String {
    match static_ability {
        StaticAbility::StaticAbility(ability) => {
            let base = serialize_standard_static_ability(ability);
            if base.ends_with('.') {
                base
            } else {
                format!("{}{}", base, strings::period_suffix())
            }
        }
        StaticAbility::WithOptions(ability) => {
            let base = serialize_standard_static_ability(&ability.ability);
            if let Some(condition) = &ability.condition {
                if matches!(condition, Condition::ThisCardIsInYourVoid) {
                    let base_no_period = base.trim_end_matches('.');
                    format!(
                        "{}{}",
                        strings::if_this_card_in_void_prefix(base_no_period),
                        strings::period_suffix()
                    )
                } else if matches!(condition, Condition::CardsInVoidCount { .. })
                    || matches!(condition, Condition::PredicateCount { count: 1, .. })
                {
                    let condition_str = condition_serializer::serialize_condition(condition);
                    let base_no_period = base.trim_end_matches('.');
                    format!(
                        "{}{}",
                        strings::condition_prepended(condition_str, base_no_period),
                        strings::period_suffix()
                    )
                } else {
                    let condition_str = condition_serializer::serialize_condition(condition);
                    let base_no_period = base.trim_end_matches('.');
                    format!(
                        "{}{}",
                        strings::condition_appended(base_no_period, condition_str),
                        strings::period_suffix()
                    )
                }
            } else if base.ends_with('.') {
                base
            } else {
                format!("{}{}", base, strings::period_suffix())
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
                predicate_serializer::serialize_card_predicate_plural(matching),
                increase.0,
            )
            .to_string()
        }
        StandardStaticAbility::YourCardsCostReduction { matching, reduction } => {
            strings::your_cards_cost_reduction(
                predicate_serializer::serialize_card_predicate_plural(matching),
                reduction.0,
            )
            .to_string()
        }
        StandardStaticAbility::EnemyCardsCostIncrease { matching, increase } => {
            strings::enemy_cards_cost_increase(
                predicate_serializer::serialize_card_predicate_plural(matching),
                increase.0,
            )
            .to_string()
        }
        StandardStaticAbility::SparkBonusOtherCharacters { matching, added_spark } => {
            strings::spark_bonus_other_characters(
                predicate_serializer::serialize_card_predicate_plural(matching),
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
                Some(CardTypeContext::Event) => "this event".to_string(),
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
            strings::once_per_turn_play_from_void(predicate_serializer::serialize_card_predicate(
                matching,
            ))
            .to_string()
        }
        StandardStaticAbility::RevealTopCardOfYourDeck => strings::reveal_top_card().to_string(),
        StandardStaticAbility::YouMayLookAtTopCardOfYourDeck => {
            strings::you_may_look_at_top_card().to_string()
        }
        StandardStaticAbility::YouMayPlayFromTopOfDeck { matching } => {
            strings::you_may_play_from_top_of_deck(
                predicate_serializer::card_predicate_base_text_plural(matching),
            )
            .to_string()
        }
        StandardStaticAbility::JudgmentTriggersWhenMaterialized { predicate } => {
            strings::judgment_triggers_when_materialized(
                predicate_serializer::serialize_predicate_plural(predicate),
            )
            .to_string()
        }
        StandardStaticAbility::SparkEqualToPredicateCount { predicate } => {
            strings::spark_equal_to_predicate_count(
                predicate_serializer::serialize_predicate_plural(predicate),
            )
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
            let predicate_text = match matching {
                ability_data::predicate::CardPredicate::Character => "allies".to_string(),
                ability_data::predicate::CardPredicate::CharacterType(subtype) => {
                    format!(
                        "allied {{@plural subtype({})}}",
                        serializer_utils::subtype_phrase_name(*subtype)
                    )
                }
                _ => predicate_serializer::serialize_card_predicate_plural(matching).to_string(),
            };
            strings::spark_bonus_your_characters(predicate_text, added_spark.0).to_string()
        }
        StandardStaticAbility::PlayFromVoid(play_from_void) => {
            if let Some(cost) = &play_from_void.additional_cost {
                let capitalized_cost =
                    strings::capitalized_sentence(cost_serializer::serialize_cost(cost))
                        .to_string();
                if let Some(effect) = &play_from_void.if_you_do {
                    let effect_text = effect_serializer::serialize_effect(effect);
                    strings::play_from_void_with_effect(
                        capitalized_cost,
                        play_from_void.energy_cost.map_or(0, |e| e.0),
                        effect_text.trim_end_matches('.'),
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
                let effect_text = effect_serializer::serialize_effect(effect);
                strings::play_from_void_for_cost_with_effect(
                    play_from_void.energy_cost.map_or(0, |e| e.0),
                    effect_text.trim_end_matches('.'),
                )
                .to_string()
            } else {
                strings::play_from_void_for_cost(play_from_void.energy_cost.map_or(0, |e| e.0))
                    .to_string()
            }
        }
    }
}
