use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::variable_value::VariableValue;
use text_formatting::FormattedText;

use crate::serializer::{serializer_utils, text_formatting};
use crate::variables::parser_bindings::VariableBindings;

pub fn serialize_predicate(predicate: &Predicate, bindings: &mut VariableBindings) -> String {
    match predicate {
        Predicate::This => "this character".to_string(),
        Predicate::That => "that character".to_string(),
        Predicate::Them => "them".to_string(),
        Predicate::It => "it".to_string(),
        Predicate::Your(card_predicate) => {
            if is_generic_card_type(card_predicate) {
                serialize_card_predicate(card_predicate, bindings)
            } else if let CardPredicate::CharacterType(subtype) = card_predicate {
                bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
                "{@a subtype($t)}".to_string()
            } else {
                // For Your predicates, use canonical form (character, not ally)
                serialize_card_predicate(card_predicate, bindings)
            }
        }
        Predicate::Another(card_predicate) => {
            // For Another predicates, use ally form
            your_predicate_formatted(card_predicate, bindings).with_article()
        }
        Predicate::Any(card_predicate) => serialize_card_predicate(card_predicate, bindings),
        Predicate::Enemy(card_predicate) => {
            enemy_predicate_formatted(card_predicate, bindings).with_article()
        }
        Predicate::YourVoid(card_predicate) => {
            format!("{} in your void", serialize_card_predicate_plural(card_predicate, bindings))
        }
        Predicate::EnemyVoid(card_predicate) => {
            format!(
                "{} in the opponent's void",
                serialize_card_predicate_plural(card_predicate, bindings)
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

pub fn serialize_predicate_plural(
    predicate: &Predicate,
    bindings: &mut VariableBindings,
) -> String {
    match predicate {
        Predicate::Another(card_predicate) => {
            serialize_your_predicate_plural(card_predicate, bindings)
        }
        Predicate::Any(card_predicate) => serialize_card_predicate_plural(card_predicate, bindings),
        Predicate::Your(card_predicate) => {
            serialize_your_predicate_plural(card_predicate, bindings)
        }
        Predicate::Enemy(card_predicate) => {
            serialize_enemy_predicate_plural(card_predicate, bindings)
        }
        Predicate::YourVoid(card_predicate) => {
            format!("{} in your void", serialize_card_predicate_plural(card_predicate, bindings))
        }
        Predicate::EnemyVoid(card_predicate) => {
            format!(
                "{} in the opponent's void",
                serialize_card_predicate_plural(card_predicate, bindings)
            )
        }
        Predicate::This => "these characters".to_string(),
        Predicate::That => "those characters".to_string(),
        Predicate::Them => "them".to_string(),
        Predicate::It => "them".to_string(),
        Predicate::AnyOther(card_predicate) => {
            format!("other {}", serialize_card_predicate_plural(card_predicate, bindings))
        }
    }
}

pub fn predicate_base_text(predicate: &Predicate, bindings: &mut VariableBindings) -> String {
    match predicate {
        Predicate::This => "this character".to_string(),
        Predicate::That => "that character".to_string(),
        Predicate::Them => "them".to_string(),
        Predicate::It => "it".to_string(),
        Predicate::Your(card_predicate) => {
            if is_generic_card_type(card_predicate) {
                text_formatting::card_predicate_base_text(card_predicate).without_article()
            } else {
                serialize_your_predicate(card_predicate, bindings)
            }
        }
        Predicate::Another(card_predicate) => serialize_your_predicate(card_predicate, bindings),
        Predicate::Any(card_predicate) => {
            text_formatting::card_predicate_base_text(card_predicate).without_article()
        }
        Predicate::Enemy(card_predicate) => {
            // For Prevent/Counterspell contexts, serialize without "enemy" prefix
            // since "enemy" is implicit for played cards being countered.
            match card_predicate {
                // Generic card types return base text without article
                _ if is_generic_card_type(card_predicate) => {
                    text_formatting::card_predicate_base_text(card_predicate).without_article()
                }
                // CouldDissolve: serialize without "enemy" prefix but include
                // the full inner target (with article)
                CardPredicate::CouldDissolve { target } => {
                    format!(
                        "event which could {{dissolve}} {}",
                        serialize_predicate(target, bindings)
                    )
                }
                // CardWithCost: serialize the card type without "enemy" prefix
                CardPredicate::CardWithCost { .. } => {
                    serialize_card_predicate_without_article(card_predicate, bindings)
                }
                // Other complex predicates: use enemy serialization
                _ => serialize_enemy_predicate(card_predicate, bindings),
            }
        }
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

pub fn serialize_your_predicate(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    your_predicate_formatted(card_predicate, bindings).without_article()
}

pub fn serialize_enemy_predicate(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    enemy_predicate_formatted(card_predicate, bindings).without_article()
}

pub fn serialize_card_predicate(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            text_formatting::card_predicate_base_text(card_predicate).with_article()
        }
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "{@a subtype($t)}".to_string()
        }
        CardPredicate::Fast { target } => {
            format!("a {{fast}} {}", serialize_fast_target(target, bindings))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            format!(
                "{} with cost {{energy($e)}}{}",
                serialize_card_predicate(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            )
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            "a character with a {materialized} ability".to_string()
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            "a character with an activated ability".to_string()
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_card_predicate(target, bindings)
            )
        }
        CardPredicate::CouldDissolve { target } => {
            format!("an event which could {{dissolve}} {}", serialize_predicate(target, bindings))
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "a character that is not {@a subtype($t)}".to_string()
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            format!(
                "a character with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            )
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            format!(
                "{} with cost less than the number of allied {}",
                serialize_card_predicate(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_card_predicate(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_card_predicate(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_card_predicate(target, bindings)
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            format!(
                "{} with cost less than the number of cards in your void",
                serialize_card_predicate(target, bindings)
            )
        }
    }
}

/// Serialize a card predicate without the article.
///
/// Use this when the caller already provides an article, like "a random".
pub fn serialize_card_predicate_without_article(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            text_formatting::card_predicate_base_text(card_predicate).without_article()
        }
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "{subtype($t)}".to_string()
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            format!(
                "{} with cost {{energy($e)}}{}",
                serialize_card_predicate_without_article(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_card_predicate_without_article(target, bindings)
            )
        }
        _ => serialize_card_predicate(card_predicate, bindings),
    }
}

/// Serialize only the cost constraint part of a card predicate.
///
/// This is used when the base card type is already implied by context (e.g.,
/// `{n_random_characters(number)}` already says "characters", so we only need
/// "with cost {energy(e)} or less").
pub fn serialize_cost_constraint_only(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            // If target is a generic type, skip it
            if is_generic_card_type(target) {
                format!(
                    "with cost {{energy($e)}}{}",
                    serializer_utils::serialize_operator(cost_operator)
                )
            } else {
                format!(
                    "{} with cost {{energy($e)}}{}",
                    serialize_card_predicate_without_article(target, bindings),
                    serializer_utils::serialize_operator(cost_operator)
                )
            }
        }
        _ => serialize_card_predicate_without_article(card_predicate, bindings),
    }
}

pub fn serialize_card_predicate_plural(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            text_formatting::card_predicate_base_text(card_predicate).plural()
        }
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "{@plural subtype($t)}".to_string()
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "characters that are not {@plural subtype($t)}".to_string()
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            format!(
                "characters with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            )
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            format!(
                "{} with cost less than the number of allied {}",
                serialize_card_predicate_plural(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_card_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_card_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_card_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            format!(
                "{} with cost less than the number of cards in your void",
                serialize_card_predicate_plural(target, bindings)
            )
        }
        CardPredicate::Fast { target } => {
            format!("fast {}", serialize_card_predicate_plural(target, bindings))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            format!(
                "{} with cost {{energy($e)}}{}",
                serialize_card_predicate_plural(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            )
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            "characters with {materialized} abilities".to_string()
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            "characters with activated abilities".to_string()
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_card_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CouldDissolve { target } => {
            format!("events which could {{dissolve}} {}", predicate_base_text(target, bindings))
        }
    }
}

pub fn serialize_fast_target(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::Card => "card".to_string(),
        CardPredicate::Character => "character".to_string(),
        CardPredicate::Event => "event".to_string(),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "{subtype($t)}".to_string()
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "character that is not {@a subtype($t)}".to_string()
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            format!("character with spark {{$s}}{}", serializer_utils::serialize_operator(operator))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            format!(
                "{} with cost less than the number of allied {}",
                serialize_fast_target(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_fast_target(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_fast_target(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_fast_target(target, bindings)
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            format!(
                "{} with cost less than the number of cards in your void",
                serialize_fast_target(target, bindings)
            )
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            format!(
                "{} with cost {{energy($e)}}{}",
                serialize_fast_target(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            )
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            "character with a {materialized} ability".to_string()
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            "character with an activated ability".to_string()
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_fast_target(target, bindings)
            )
        }
        CardPredicate::CouldDissolve { target } => {
            format!("event which could {{dissolve}} {}", predicate_base_text(target, bindings))
        }
        CardPredicate::Fast { target } => {
            format!("fast {}", serialize_fast_target(target, bindings))
        }
    }
}

pub fn serialize_for_each_predicate(
    predicate: &Predicate,
    bindings: &mut VariableBindings,
) -> String {
    match predicate {
        Predicate::Another(CardPredicate::Character) => "allied character".to_string(),
        Predicate::Another(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "allied {subtype($t)}".to_string()
        }
        Predicate::Your(CardPredicate::Character) => "ally".to_string(),
        Predicate::Your(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "allied {subtype($t)}".to_string()
        }
        Predicate::Enemy(CardPredicate::Character) => "enemy".to_string(),
        Predicate::Any(CardPredicate::Character) => "character".to_string(),
        Predicate::Any(CardPredicate::Card) => "card".to_string(),
        Predicate::YourVoid(CardPredicate::Card) => "card in your void".to_string(),
        Predicate::This => "this character".to_string(),
        Predicate::It => "that character".to_string(),
        Predicate::That => "that character".to_string(),
        Predicate::Them => "character".to_string(),
        Predicate::Enemy(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "enemy {subtype($t)}".to_string()
        }
        Predicate::Any(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "{subtype($t)}".to_string()
        }
        Predicate::Any(CardPredicate::Event) => "event".to_string(),
        Predicate::AnyOther(CardPredicate::Character) => "other character".to_string(),
        Predicate::AnyOther(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "other {subtype($t)}".to_string()
        }
        Predicate::YourVoid(CardPredicate::Character) => "character in your void".to_string(),
        Predicate::YourVoid(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "{subtype($t)} in your void".to_string()
        }
        Predicate::YourVoid(CardPredicate::Event) => "event in your void".to_string(),
        Predicate::EnemyVoid(CardPredicate::Card) => "card in the opponent's void".to_string(),
        Predicate::EnemyVoid(CardPredicate::Character) => {
            "character in the opponent's void".to_string()
        }
        Predicate::EnemyVoid(CardPredicate::Event) => "event in the opponent's void".to_string(),
        Predicate::Your(CardPredicate::Event) => "allied event".to_string(),
        Predicate::Another(CardPredicate::CharacterWithSpark(spark, operator)) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            format!("ally with spark {{$s}}{}", serializer_utils::serialize_operator(operator))
        }
        Predicate::Your(CardPredicate::CharacterWithSpark(spark, operator)) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            format!("ally with spark {{$s}}{}", serializer_utils::serialize_operator(operator))
        }
        predicate => format!("each {}", predicate_base_text(predicate, bindings)),
    }
}

fn your_predicate_formatted(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> FormattedText {
    match card_predicate {
        CardPredicate::Character => FormattedText::new("ally"),
        CardPredicate::Card => FormattedText::new("your card"),
        CardPredicate::Event => FormattedText::new("your event"),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            FormattedText::new("allied {subtype($t)}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            FormattedText::new("ally that is not {@a subtype($t)}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            FormattedText::new(&format!(
                "ally with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            FormattedText::new("ally with a {materialized} ability")
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            FormattedText::new("ally with an activated ability")
        }
        CardPredicate::Fast { target } => {
            FormattedText::new(&format!("{{fast}} {}", serialize_fast_target(target, bindings)))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            FormattedText::new(&format!(
                "{} with cost {{energy($e)}}{}",
                serialize_your_predicate(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            ))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            FormattedText::new(&format!(
                "{} with cost less than the number of allied {}",
                serialize_your_predicate(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            FormattedText::new(&format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            FormattedText::new(&format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            FormattedText::new(&format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            FormattedText::new(&format!(
                "{} with cost less than the number of cards in your void",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            FormattedText::new(&format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CouldDissolve { target } => FormattedText::new(&format!(
            "your event which could {{dissolve}} {}",
            predicate_base_text(target, bindings)
        )),
    }
}

fn enemy_predicate_formatted(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> FormattedText {
    match card_predicate {
        CardPredicate::Character => FormattedText::new("enemy"),
        CardPredicate::Card => FormattedText::new("enemy card"),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            FormattedText::new("enemy {subtype($t)}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            FormattedText::new_non_vowel("non-{subtype($t)} enemy")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            FormattedText::new(&format!(
                "enemy with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            FormattedText::new("enemy with a {materialized} ability")
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            FormattedText::new("enemy with an activated ability")
        }
        CardPredicate::CardWithCost { cost_operator, cost, .. } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            FormattedText::new(&format!(
                "enemy with cost {{energy($e)}}{}",
                serializer_utils::serialize_operator(cost_operator)
            ))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            FormattedText::new(&format!(
                "{} with cost less than the number of allied {}",
                serialize_enemy_predicate(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            FormattedText::new(&format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            FormattedText::new(&format!(
                "{} with cost less than the number of cards in your void",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            FormattedText::new(&format!(
                "{} with spark less than that ally's spark",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            FormattedText::new(&format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            FormattedText::new(&format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::Fast { target } => {
            FormattedText::new(&format!("{{fast}} {}", serialize_fast_target(target, bindings)))
        }
        CardPredicate::Event => FormattedText::new("enemy event"),
        CardPredicate::CouldDissolve { target } => FormattedText::new(&format!(
            "event which could {{dissolve}} {}",
            serialize_predicate(target, bindings)
        )),
    }
}

fn serialize_your_predicate_plural(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::Character => "allies".to_string(),
        CardPredicate::Card => "your cards".to_string(),
        CardPredicate::Event => "your events".to_string(),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "allied {@plural subtype($t)}".to_string()
        }
        CardPredicate::Fast { target } => {
            format!("{{fast}} {}", serialize_card_predicate_plural(target, bindings))
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "allies that are not {@plural subtype($t)}".to_string()
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            format!("allies with spark {{$s}}{}", serializer_utils::serialize_operator(operator))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            "allies with {materialized} abilities".to_string()
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            "allies with activated abilities".to_string()
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            format!(
                "{} with cost {{energy($e)}}{}",
                serialize_your_predicate_plural(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            )
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            format!(
                "{} with cost less than the number of allied {}",
                serialize_your_predicate_plural(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_your_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_your_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_your_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            format!(
                "{} with cost less than the number of cards in your void",
                serialize_your_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_your_predicate_plural(target, bindings)
            )
        }
        CardPredicate::CouldDissolve { target } => {
            format!(
                "your events which could {{dissolve}} {}",
                predicate_base_text(target, bindings)
            )
        }
    }
}

fn serialize_enemy_predicate_plural(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::Character => "enemies".to_string(),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "enemy {@plural subtype($t)}".to_string()
        }
        _ => {
            format!("enemy {}", serialize_card_predicate_plural(card_predicate, bindings))
        }
    }
}

/// Returns true if the card predicate is a generic type (Card, Character,
/// Event) without any modifiers. These are serialized without ownership
/// qualifiers for round-trip compatibility.
fn is_generic_card_type(card_predicate: &CardPredicate) -> bool {
    matches!(card_predicate, CardPredicate::Card | CardPredicate::Character | CardPredicate::Event)
}
