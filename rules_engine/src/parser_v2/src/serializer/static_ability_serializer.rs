use ability_data::static_ability::{StandardStaticAbility, StaticAbility};

use super::predicate_serializer::serialize_card_predicate_plural;

pub fn serialize_static_ability(static_ability: &StaticAbility) -> String {
    match static_ability {
        StaticAbility::StaticAbility(ability) => serialize_standard_static_ability(ability),
        StaticAbility::WithOptions(ability) => {
            if ability.condition.is_none() {
                serialize_standard_static_ability(&ability.ability)
            } else {
                unimplemented!("Serialization not yet implemented for this static ability")
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
        _ => unimplemented!("Serialization not yet implemented for this static ability"),
    }
}
