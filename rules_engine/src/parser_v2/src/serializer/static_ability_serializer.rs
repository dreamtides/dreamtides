use ability_data::condition::Condition;
use ability_data::static_ability::{StandardStaticAbility, StaticAbility};

use super::{cost_serializer, predicate_serializer, serializer_utils, text_formatting};

pub fn serialize_static_ability(static_ability: &StaticAbility) -> String {
    match static_ability {
        StaticAbility::StaticAbility(ability) => {
            let base = serialize_standard_static_ability(ability);
            if base.ends_with('.') {
                base
            } else {
                format!("{}.", base)
            }
        }
        StaticAbility::WithOptions(ability) => {
            let base = serialize_standard_static_ability(&ability.ability);
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
                    let condition_str = serialize_condition(condition);
                    if base.ends_with('.') {
                        format!("{} {}", condition_str, base)
                    } else {
                        format!("{} {}.", condition_str, base)
                    }
                } else {
                    let condition_str = serialize_condition(condition);
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

pub fn serialize_standard_static_ability(ability: &StandardStaticAbility) -> String {
    match ability {
        StandardStaticAbility::YourCardsCostIncrease { matching, .. } => {
            format!(
                "{} cost you {{e}} more.",
                predicate_serializer::serialize_card_predicate_plural(matching)
            )
        }
        StandardStaticAbility::YourCardsCostReduction { matching, .. } => {
            format!(
                "{} cost you {{e}} less.",
                predicate_serializer::serialize_card_predicate_plural(matching)
            )
        }
        StandardStaticAbility::EnemyCardsCostIncrease { matching, .. } => {
            format!(
                "the opponent's {} cost {{e}} more.",
                predicate_serializer::serialize_card_predicate_plural(matching)
            )
        }
        StandardStaticAbility::SparkBonusOtherCharacters { matching, .. } => {
            format!(
                "allied {} have +{{s}} spark.",
                predicate_serializer::serialize_card_predicate_plural(matching)
            )
        }
        StandardStaticAbility::AdditionalCostToPlay(cost) => {
            format!("To play this card, {}.", cost_serializer::serialize_cost(cost))
        }
        StandardStaticAbility::PlayForAlternateCost(alt_cost) => {
            if let Some(cost) = &alt_cost.additional_cost {
                let card_type = if alt_cost.if_you_do.is_some() { "character" } else { "event" };
                let base = format!(
                    "{}: Play this {} for {{e}}",
                    serializer_utils::capitalize_first_letter(&cost_serializer::serialize_cost(
                        cost
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
        StandardStaticAbility::MultiplyEnergyGainFromCardEffects { .. } => {
            "{multiplyby} the amount of {energy-symbol} you gain from card effects this turn."
                .to_string()
        }
        StandardStaticAbility::MultiplyCardDrawFromCardEffects { .. } => {
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
                predicate_serializer::predicate_base_text(predicate)
            )
        }
        StandardStaticAbility::SparkEqualToPredicateCount { predicate } => {
            format!(
                "this character's spark is equal to the number of {}.",
                predicate_serializer::predicate_base_text(predicate)
            )
        }
        StandardStaticAbility::PlayOnlyFromVoid => {
            "you may only play this character from your void.".to_string()
        }
        StandardStaticAbility::PlayFromHandOrVoidForCost(_) => {
            "you may play this card from your hand or void for {e}".to_string()
        }
        StandardStaticAbility::CardsInYourVoidHaveReclaim { .. } => {
            "they have {reclaim} equal to their cost.".to_string()
        }
        _ => unimplemented!("Serialization not yet implemented for this static ability"),
    }
}

fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::DissolvedThisTurn { .. } => "if a character dissolved this turn".to_string(),
        Condition::CardsDiscardedThisTurn { count: 1, predicate } => {
            format!(
                "if you have discarded {} this turn",
                predicate_serializer::serialize_card_predicate(predicate)
            )
        }
        Condition::CardsInVoidCount { .. } => {
            "while you have {count} or more cards in your void,".to_string()
        }
        Condition::PredicateCount { count: 1, .. } => "with an allied {subtype},".to_string(),
        _ => unimplemented!("Serialization not yet implemented for this condition type"),
    }
}
