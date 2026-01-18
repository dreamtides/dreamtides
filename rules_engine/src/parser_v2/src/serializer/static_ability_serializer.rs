use ability_data::condition::Condition;
use ability_data::static_ability::{StandardStaticAbility, StaticAbility};
use ability_data::variable_value::VariableValue;

use crate::parser_v2::serializer::{
    condition_serializer, cost_serializer, effect_serializer, predicate_serializer,
    serializer_utils, text_formatting,
};
use crate::variables::parser_bindings::VariableBindings;
use crate::variables::parser_substitutions;
pub fn serialize_static_ability(
    static_ability: &StaticAbility,
    bindings: &mut VariableBindings,
) -> String {
    match static_ability {
        StaticAbility::StaticAbility(ability) => {
            let base = serialize_standard_static_ability(ability, bindings);
            if base.ends_with('.') {
                base
            } else {
                format!("{}.", base)
            }
        }
        StaticAbility::WithOptions(ability) => {
            let base = serialize_standard_static_ability(&ability.ability, bindings);
            if let Some(condition) = &ability.condition {
                if matches!(condition, Condition::ThisCardIsInYourVoid) {
                    if base.ends_with('.') {
                        format!("while this card is in your void, {}", base)
                    } else {
                        format!("while this card is in your void, {}.", base)
                    }
                } else if matches!(condition, Condition::CardsInVoidCount { .. })
                    || matches!(condition, Condition::PredicateCount { count: 1, .. })
                {
                    let condition_str =
                        condition_serializer::serialize_condition(condition, bindings);
                    if base.ends_with('.') {
                        format!("{} {}", condition_str, base)
                    } else {
                        format!("{} {}.", condition_str, base)
                    }
                } else {
                    let condition_str =
                        condition_serializer::serialize_condition(condition, bindings);
                    if base.ends_with('.') {
                        format!("{} {}.", base.trim_end_matches('.'), condition_str)
                    } else {
                        format!("{} {}.", base, condition_str)
                    }
                }
            } else if base.ends_with('.') {
                base
            } else {
                format!("{}.", base)
            }
        }
    }
}
pub fn serialize_standard_static_ability(
    ability: &StandardStaticAbility,
    bindings: &mut VariableBindings,
) -> String {
    match ability {
        StandardStaticAbility::YourCardsCostIncrease { matching, increase } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(increase.0));
            }
            format!(
                "{} cost you {{e}} more.",
                predicate_serializer::serialize_card_predicate_plural(matching, bindings)
            )
        }
        StandardStaticAbility::YourCardsCostReduction { matching, reduction } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(reduction.0));
            }
            format!(
                "{} cost you {{e}} less.",
                predicate_serializer::serialize_card_predicate_plural(matching, bindings)
            )
        }
        StandardStaticAbility::EnemyCardsCostIncrease { matching, increase } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(increase.0));
            }
            format!(
                "the opponent's {} cost {{e}} more.",
                predicate_serializer::serialize_card_predicate_plural(matching, bindings)
            )
        }
        StandardStaticAbility::SparkBonusOtherCharacters { matching, added_spark } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("s") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(added_spark.0));
            }
            format!(
                "allied {} have +{{s}} spark.",
                predicate_serializer::serialize_card_predicate_plural(matching, bindings)
            )
        }
        StandardStaticAbility::AdditionalCostToPlay(cost) => {
            format!("To play this card, {}.", cost_serializer::serialize_cost(cost, bindings))
        }
        StandardStaticAbility::PlayForAlternateCost(alt_cost) => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
                bindings
                    .insert(var_name.to_string(), VariableValue::Integer(alt_cost.energy_cost.0));
            }
            if let Some(cost) = &alt_cost.additional_cost {
                let card_type = if alt_cost.if_you_do.is_some() { "character" } else { "event" };
                let base = format!(
                    "{}: Play this {} for {{e}}",
                    serializer_utils::capitalize_first_letter(&cost_serializer::serialize_cost(
                        cost, bindings,
                    )),
                    card_type
                );
                if alt_cost.if_you_do.is_some() {
                    format!("{}, then abandon it.", base)
                } else {
                    format!("{}.", base)
                }
            } else {
                "this event costs {e}".to_string()
            }
        }
        StandardStaticAbility::CharactersInHandHaveFast => {
            "characters in your hand have {fast}.".to_string()
        }
        StandardStaticAbility::DisableEnemyMaterializedAbilities => {
            "disable the {Materialized} abilities of enemies.".to_string()
        }
        StandardStaticAbility::HasAllCharacterTypes => "has all character types.".to_string(),
        StandardStaticAbility::MultiplyEnergyGainFromCardEffects { multiplier } => {
            if let Some(var_name) =
                parser_substitutions::directive_to_integer_variable("multiplyby")
            {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*multiplier));
            }
            "{multiplyby} the amount of {energy-symbol} you gain from card effects this turn."
                .to_string()
        }
        StandardStaticAbility::MultiplyCardDrawFromCardEffects { multiplier } => {
            if let Some(var_name) =
                parser_substitutions::directive_to_integer_variable("multiplyby")
            {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*multiplier));
            }
            "{multiplyby} the number of cards you draw from card effects this turn.".to_string()
        }
        StandardStaticAbility::OncePerTurnPlayFromVoid { matching } => {
            format!(
                "once per turn, you may play {} from your void.",
                text_formatting::card_predicate_base_text(matching).without_article()
            )
        }
        StandardStaticAbility::RevealTopCardOfYourDeck => {
            "reveal the top card of your deck.".to_string()
        }
        StandardStaticAbility::YouMayLookAtTopCardOfYourDeck => {
            "you may look at the top card of your deck.".to_string()
        }
        StandardStaticAbility::YouMayPlayFromTopOfDeck { matching } => {
            format!(
                "you may play {} from the top of your deck.",
                text_formatting::card_predicate_base_text(matching).without_article()
            )
        }
        StandardStaticAbility::JudgmentTriggersWhenMaterialized { predicate } => {
            format!(
                "the '{{Judgment}}' ability of {} triggers when you {{materialize}} them.",
                predicate_serializer::predicate_base_text(predicate, bindings)
            )
        }
        StandardStaticAbility::SparkEqualToPredicateCount { predicate } => {
            format!(
                "this character's spark is equal to the number of {}.",
                predicate_serializer::predicate_base_text(predicate, bindings)
            )
        }
        StandardStaticAbility::PlayOnlyFromVoid => {
            "you may only play this character from your void.".to_string()
        }
        StandardStaticAbility::PlayFromHandOrVoidForCost(play_from_hand_or_void) => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
                bindings.insert(
                    var_name.to_string(),
                    VariableValue::Integer(play_from_hand_or_void.energy_cost.0),
                );
            }
            "you may play this card from your hand or void for {e}".to_string()
        }
        StandardStaticAbility::CardsInYourVoidHaveReclaim { .. } => {
            "they have {reclaim} equal to their cost.".to_string()
        }
        StandardStaticAbility::CostReductionForEach { reduction, quantity } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(reduction.0));
            }
            format!(
                "this card costs {{e}} less for each {}.",
                effect_serializer::serialize_for_count_expression(quantity, bindings)
            )
        }
        StandardStaticAbility::SparkBonusYourCharacters { matching, added_spark } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable("s") {
                bindings.insert(var_name.to_string(), VariableValue::Integer(added_spark.0));
            }
            let predicate_text = match matching {
                ability_data::predicate::CardPredicate::Character => "allies".to_string(),
                ability_data::predicate::CardPredicate::CharacterType(_) => {
                    "allied {plural-subtype}".to_string()
                }
                _ => predicate_serializer::serialize_card_predicate_plural(matching, bindings),
            };
            format!("{} have +{{s}} spark.", predicate_text)
        }
        StandardStaticAbility::PlayFromVoid(play_from_void) => {
            if let Some(energy_cost) = play_from_void.energy_cost {
                if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
                    bindings.insert(var_name.to_string(), VariableValue::Integer(energy_cost.0));
                }
            }
            let mut result = String::new();
            if let Some(cost) = &play_from_void.additional_cost {
                result.push_str(&format!(
                    "{}: ",
                    serializer_utils::capitalize_first_letter(&cost_serializer::serialize_cost(
                        cost, bindings,
                    ))
                ));
            }
            result.push_str("play this card from your void for {e}");
            if let Some(effect) = &play_from_void.if_you_do {
                let effect_text = effect_serializer::serialize_effect(effect, bindings);
                result.push_str(&format!(", then {}", effect_text.trim_end_matches('.')));
            }
            result.push('.');
            result
        }
    }
}
