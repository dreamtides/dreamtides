use ability_data::predicate::{CardPredicate, Predicate};

use super::{serializer_utils, text_formatting};

pub fn serialize_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::This => "this character".to_string(),
        Predicate::That => "that character".to_string(),
        Predicate::Them => "them".to_string(),
        Predicate::It => "it".to_string(),
        Predicate::Your(card_predicate) => {
            format!("an {}", serialize_your_predicate(card_predicate))
        }
        Predicate::Another(card_predicate) => {
            format!("an {}", serialize_your_predicate(card_predicate))
        }
        Predicate::Any(card_predicate) => serialize_card_predicate(card_predicate),
        Predicate::Enemy(card_predicate) => {
            format!("an {}", serialize_enemy_predicate(card_predicate))
        }
        Predicate::YourVoid(card_predicate) => {
            format!("{} in your void", serialize_card_predicate_plural(card_predicate))
        }
        Predicate::EnemyVoid(card_predicate) => {
            format!("{} in the opponent's void", serialize_card_predicate_plural(card_predicate))
        }
        Predicate::AnyOther(card_predicate) => {
            format!(
                "another {}",
                text_formatting::card_predicate_base_text(card_predicate).without_article()
            )
        }
    }
}

pub fn serialize_predicate_plural(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Another(card_predicate) => serialize_your_predicate_plural(card_predicate),
        Predicate::Any(card_predicate) => serialize_card_predicate_plural(card_predicate),
        Predicate::Your(card_predicate) => serialize_your_predicate_plural(card_predicate),
        Predicate::Enemy(card_predicate) => serialize_enemy_predicate_plural(card_predicate),
        Predicate::YourVoid(card_predicate) => {
            format!("{} in your void", serialize_card_predicate_plural(card_predicate))
        }
        Predicate::EnemyVoid(card_predicate) => {
            format!("{} in the opponent's void", serialize_card_predicate_plural(card_predicate))
        }
        _ => unimplemented!("Serialization not yet implemented for this plural predicate type"),
    }
}

pub fn predicate_base_text(predicate: &Predicate) -> String {
    match predicate {
        Predicate::This => "this character".to_string(),
        Predicate::That => "that character".to_string(),
        Predicate::Them => "them".to_string(),
        Predicate::It => "it".to_string(),
        Predicate::Your(card_predicate) => serialize_your_predicate(card_predicate),
        Predicate::Another(card_predicate) => serialize_your_predicate(card_predicate),
        Predicate::Any(card_predicate) => {
            text_formatting::card_predicate_base_text(card_predicate).without_article()
        }
        Predicate::Enemy(card_predicate) => serialize_enemy_predicate(card_predicate),
        Predicate::YourVoid(card_predicate) => {
            format!(
                "{} in your void",
                text_formatting::card_predicate_base_text(card_predicate).plural()
            )
        }
        Predicate::EnemyVoid(card_predicate) => {
            format!(
                "{} in the opponent's void",
                text_formatting::card_predicate_base_text(card_predicate).plural()
            )
        }
        Predicate::AnyOther(card_predicate) => {
            format!(
                "another {}",
                text_formatting::card_predicate_base_text(card_predicate).without_article()
            )
        }
    }
}

pub fn serialize_your_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => "ally".to_string(),
        CardPredicate::Card => "your card".to_string(),
        CardPredicate::Event => "your event".to_string(),
        CardPredicate::CharacterType(_) => "allied {subtype}".to_string(),
        CardPredicate::NotCharacterType(_) => "ally that is not {a-subtype}".to_string(),
        CardPredicate::CharacterWithSpark(_, operator) => {
            format!("ally with spark {{s}} {}", serializer_utils::serialize_operator(operator))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            "ally with a {materialized} ability".to_string()
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            "ally with an activated ability".to_string()
        }
        CardPredicate::Fast { target } => {
            format!("fast {}", serialize_your_predicate(target))
        }
        CardPredicate::CardWithCost { target, cost_operator, .. } => {
            format!(
                "{} with cost {{e}} {}",
                serialize_your_predicate(target),
                serializer_utils::serialize_operator(cost_operator)
            )
        }
        _ => {
            unimplemented!("Serialization not yet implemented for this your predicate type")
        }
    }
}

pub fn serialize_enemy_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => "enemy".to_string(),
        CardPredicate::Card => "enemy card".to_string(),
        CardPredicate::CharacterType(_) => "enemy {subtype}".to_string(),
        CardPredicate::NotCharacterType(_) => "enemy that is not {a-subtype}".to_string(),
        CardPredicate::CharacterWithSpark(_, operator) => {
            format!("enemy with spark {{s}} {}", serializer_utils::serialize_operator(operator))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            "enemy with a {materialized} ability".to_string()
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            "enemy with an activated ability".to_string()
        }
        CardPredicate::CardWithCost { cost_operator, .. } => {
            format!("enemy with cost {{e}} {}", serializer_utils::serialize_operator(cost_operator))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            format!(
                "{} with cost less than the number of allied {}",
                serialize_enemy_predicate(target),
                serialize_card_predicate_plural(count_matching)
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_enemy_predicate(target)
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            format!(
                "{} with cost less than the number of cards in your void",
                serialize_enemy_predicate(target)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            format!("{} with spark less than that ally's spark", serialize_enemy_predicate(target))
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_enemy_predicate(target)
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            format!(
                "{} with spark less than the amount of {{energy-symbol}} paid",
                serialize_enemy_predicate(target)
            )
        }
        CardPredicate::Fast { target } => {
            format!("fast {}", serialize_enemy_predicate(target))
        }
        _ => {
            unimplemented!("Serialization not yet implemented for this enemy predicate type")
        }
    }
}

pub fn serialize_card_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            text_formatting::card_predicate_base_text(card_predicate).with_article()
        }
        CardPredicate::CharacterType(_) => "{a-subtype}".to_string(),
        CardPredicate::Fast { target } => {
            format!("a {{fast}} {}", serialize_fast_target(target))
        }
        CardPredicate::CardWithCost { target, cost_operator, .. } => format!(
            "{} with cost {{e}} {}",
            serialize_card_predicate(target),
            serializer_utils::serialize_operator(cost_operator)
        ),
        CardPredicate::CharacterWithMaterializedAbility => {
            "a character with a {materialized} ability".to_string()
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            "a character with an activated ability".to_string()
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            format!(
                "{} with spark less than the amount of {{energy-symbol}} paid",
                serialize_card_predicate(target)
            )
        }
        CardPredicate::CouldDissolve { target } => {
            format!("an event which could {{dissolve}} {}", predicate_base_text(target))
        }
        _ => {
            unimplemented!("Serialization not yet implemented for this card predicate type")
        }
    }
}

pub fn serialize_card_predicate_plural(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            text_formatting::card_predicate_base_text(card_predicate).plural()
        }
        CardPredicate::CharacterType(_) => "{plural-subtype}".to_string(),
        CardPredicate::Fast { target } => {
            format!("fast {}", serialize_card_predicate_plural(target))
        }
        _ => {
            unimplemented!("Serialization not yet implemented for this card predicate type")
        }
    }
}

pub fn serialize_fast_target(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Card => "card".to_string(),
        CardPredicate::Character => "character".to_string(),
        CardPredicate::Event => "event".to_string(),
        CardPredicate::CharacterType(_) => "{subtype}".to_string(),
        CardPredicate::CharacterWithSpark(_spark, operator) => {
            format!("character with spark {{s}} {}", serializer_utils::serialize_operator(operator))
        }
        CardPredicate::CardWithCost { target, cost_operator, .. } => {
            format!(
                "{} with cost {{e}} {}",
                serialize_fast_target(target),
                serializer_utils::serialize_operator(cost_operator)
            )
        }
        _ => unimplemented!("Unsupported fast target"),
    }
}

pub fn serialize_for_each_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Another(CardPredicate::Character) => "allied character".to_string(),
        Predicate::Another(CardPredicate::CharacterType(_)) => "allied {subtype}".to_string(),
        _ => {
            unimplemented!("Serialization not yet implemented for this for-each predicate")
        }
    }
}

fn serialize_your_predicate_plural(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => "allies".to_string(),
        CardPredicate::Card => "your cards".to_string(),
        CardPredicate::Event => "your events".to_string(),
        CardPredicate::CharacterType(_) => "allied {plural-subtype}".to_string(),
        CardPredicate::Fast { target } => {
            format!("allied fast {}", serialize_card_predicate_plural(target))
        }
        _ => unimplemented!("Serialization not yet implemented for this allied plural predicate"),
    }
}

fn serialize_enemy_predicate_plural(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => "enemies".to_string(),
        CardPredicate::CharacterType(_) => "enemy {plural-subtype}".to_string(),
        _ => format!("enemy {}", serialize_card_predicate_plural(card_predicate)),
    }
}
