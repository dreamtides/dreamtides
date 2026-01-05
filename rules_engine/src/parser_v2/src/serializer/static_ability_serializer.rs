use ability_data::condition::Condition;
use ability_data::static_ability::{StandardStaticAbility, StaticAbility};

use super::cost_serializer::serialize_cost;
use super::predicate_serializer::{
    serialize_card_predicate_plural, serialize_card_predicate_without_article,
    serialize_predicate_without_article,
};
use super::serializer_utils::capitalize_first_letter;

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
                let condition_str = serialize_condition(condition);
                if base.ends_with('.') {
                    format!("{} {}.", base.trim_end_matches('.'), condition_str)
                } else {
                    format!("{} {}.", base, condition_str)
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
            format!("{} cost you {{e}} more.", serialize_card_predicate_plural(matching))
        }
        StandardStaticAbility::YourCardsCostReduction { matching, .. } => {
            format!("{} cost you {{e}} less.", serialize_card_predicate_plural(matching))
        }
        StandardStaticAbility::EnemyCardsCostIncrease { matching, .. } => {
            format!("the opponent's {} cost {{e}} more.", serialize_card_predicate_plural(matching))
        }
        StandardStaticAbility::SparkBonusOtherCharacters { matching, .. } => {
            format!("allied {} have +{{s}} spark.", serialize_card_predicate_plural(matching))
        }
        StandardStaticAbility::AdditionalCostToPlay(cost) => {
            format!("To play this card, {}.", serialize_cost(cost))
        }
        StandardStaticAbility::PlayForAlternateCost(alt_cost) => {
            if let Some(cost) = &alt_cost.additional_cost {
                let card_type = if alt_cost.if_you_do.is_some() { "character" } else { "event" };
                let base = format!(
                    "{}: Play this {} for {{e}}",
                    capitalize_first_letter(&serialize_cost(cost)),
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
                serialize_card_predicate_without_article(matching)
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
                serialize_card_predicate_without_article(matching)
            )
        }
        StandardStaticAbility::JudgmentTriggersWhenMaterialized { predicate } => {
            format!(
                "the '{{Judgment}}' ability of {} triggers when you {{materialize}} them.",
                serialize_predicate_without_article(predicate)
            )
        }
        StandardStaticAbility::SparkEqualToPredicateCount { predicate } => {
            format!(
                "this character's spark is equal to the number of {}.",
                serialize_predicate_without_article(predicate)
            )
        }
        _ => unimplemented!("Serialization not yet implemented for this static ability"),
    }
}

fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::DissolvedThisTurn { .. } => "if a character dissolved this turn".to_string(),
        Condition::CardsDiscardedThisTurn { count: 1 } => {
            "if you have discarded a card this turn".to_string()
        }
        _ => unimplemented!("Serialization not yet implemented for this condition type"),
    }
}
