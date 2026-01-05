use ability_data::condition::Condition;
use ability_data::static_ability::{StandardStaticAbility, StaticAbility};

use super::cost_serializer::serialize_cost;
use super::predicate_serializer::serialize_card_predicate_plural;
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
        _ => unimplemented!("Serialization not yet implemented for this static ability"),
    }
}

fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::DissolvedThisTurn { .. } => "if a character dissolved this turn".to_string(),
        _ => unimplemented!("Serialization not yet implemented for this condition type"),
    }
}
