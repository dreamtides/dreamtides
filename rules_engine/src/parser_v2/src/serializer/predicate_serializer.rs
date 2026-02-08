use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::variable_value::VariableValue;
use rlf::{Phrase, Tag, VariantKey};
use strings::strings;

use crate::serializer::serializer_utils;
use crate::variables::parser_bindings::VariableBindings;

/// Serializes a predicate to a Phrase.
pub fn serialize_predicate(predicate: &Predicate, bindings: &mut VariableBindings) -> Phrase {
    match predicate {
        Predicate::This => text_phrase("this character"),
        Predicate::That => text_phrase("that character"),
        Predicate::Them => text_phrase("them"),
        Predicate::It => text_phrase("it"),
        Predicate::Your(card_predicate) => {
            if is_generic_card_type(card_predicate) {
                serialize_card_predicate(card_predicate, bindings)
            } else if let CardPredicate::CharacterType(subtype) = card_predicate {
                bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
                text_phrase("{@a subtype($t)}")
            } else {
                serialize_card_predicate(card_predicate, bindings)
            }
        }
        Predicate::Another(card_predicate) => {
            text_phrase(&with_article(&your_predicate_formatted(card_predicate, bindings)))
        }
        Predicate::Any(card_predicate) => serialize_card_predicate(card_predicate, bindings),
        Predicate::Enemy(card_predicate) => {
            text_phrase(&with_article(&enemy_predicate_formatted(card_predicate, bindings)))
        }
        Predicate::YourVoid(card_predicate) => text_phrase(&format!(
            "{} in your void",
            serialize_card_predicate_plural(card_predicate, bindings)
        )),
        Predicate::EnemyVoid(card_predicate) => text_phrase(&format!(
            "{} in the opponent's void",
            serialize_card_predicate_plural(card_predicate, bindings)
        )),
        Predicate::AnyOther(card_predicate) => {
            text_phrase(&format!("another {}", card_predicate_base_phrase(card_predicate)))
        }
    }
}

/// Serializes a predicate in plural form.
pub fn serialize_predicate_plural(
    predicate: &Predicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    match predicate {
        Predicate::Another(card_predicate) => {
            text_phrase(&serialize_your_predicate_plural(card_predicate, bindings))
        }
        Predicate::Any(card_predicate) => serialize_card_predicate_plural(card_predicate, bindings),
        Predicate::Your(card_predicate) => {
            text_phrase(&serialize_your_predicate_plural(card_predicate, bindings))
        }
        Predicate::Enemy(card_predicate) => {
            text_phrase(&serialize_enemy_predicate_plural(card_predicate, bindings))
        }
        Predicate::YourVoid(card_predicate) => text_phrase(&format!(
            "{} in your void",
            serialize_card_predicate_plural(card_predicate, bindings)
        )),
        Predicate::EnemyVoid(card_predicate) => text_phrase(&format!(
            "{} in the opponent's void",
            serialize_card_predicate_plural(card_predicate, bindings)
        )),
        Predicate::This => text_phrase("these characters"),
        Predicate::That => text_phrase("those characters"),
        Predicate::Them => text_phrase("them"),
        Predicate::It => text_phrase("them"),
        Predicate::AnyOther(card_predicate) => text_phrase(&format!(
            "other {}",
            serialize_card_predicate_plural(card_predicate, bindings)
        )),
    }
}

/// Serializes a predicate to its base text without an article.
pub fn predicate_base_text(predicate: &Predicate, bindings: &mut VariableBindings) -> Phrase {
    match predicate {
        Predicate::This => text_phrase("this character"),
        Predicate::That => text_phrase("that character"),
        Predicate::Them => text_phrase("them"),
        Predicate::It => text_phrase("it"),
        Predicate::Your(card_predicate) => {
            if is_generic_card_type(card_predicate) {
                card_predicate_base_phrase(card_predicate)
            } else {
                serialize_your_predicate(card_predicate, bindings)
            }
        }
        Predicate::Another(card_predicate) => serialize_your_predicate(card_predicate, bindings),
        Predicate::Any(card_predicate) => card_predicate_base_phrase(card_predicate),
        Predicate::Enemy(card_predicate) => match card_predicate {
            _ if is_generic_card_type(card_predicate) => card_predicate_base_phrase(card_predicate),
            CardPredicate::CouldDissolve { target } => text_phrase(&format!(
                "event which could {{dissolve}} {}",
                serialize_predicate(target, bindings)
            )),
            CardPredicate::CardWithCost { .. } => {
                serialize_card_predicate_without_article(card_predicate, bindings)
            }
            _ => serialize_enemy_predicate(card_predicate, bindings),
        },
        Predicate::YourVoid(card_predicate) => text_phrase(&format!(
            "{} in your void",
            phrase_plural(&card_predicate_base_phrase(card_predicate))
        )),
        Predicate::EnemyVoid(card_predicate) => text_phrase(&format!(
            "{} in the opponent's void",
            phrase_plural(&card_predicate_base_phrase(card_predicate))
        )),
        Predicate::AnyOther(card_predicate) => {
            text_phrase(&format!("another {}", card_predicate_base_phrase(card_predicate)))
        }
    }
}

/// Serializes a "your" card predicate to a Phrase.
pub fn serialize_your_predicate(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    your_predicate_formatted(card_predicate, bindings)
}

/// Serializes an "enemy" card predicate to a Phrase.
pub fn serialize_enemy_predicate(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    enemy_predicate_formatted(card_predicate, bindings)
}

/// Serializes a card predicate with an article.
pub fn serialize_card_predicate(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            text_phrase(&with_article(&card_predicate_base_phrase(card_predicate)))
        }
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("{@a subtype($t)}")
        }
        CardPredicate::Fast { target } => {
            text_phrase(&format!("a {{fast}} {}", serialize_fast_target(target, bindings)))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            text_phrase(&format!(
                "{} with cost {{energy($e)}}{}",
                serialize_card_predicate(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            text_phrase("a character with a {materialized} ability")
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            text_phrase("a character with an activated ability")
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_card_predicate(target, bindings)
            ))
        }
        CardPredicate::CouldDissolve { target } => text_phrase(&format!(
            "an event which could {{dissolve}} {}",
            serialize_predicate(target, bindings)
        )),
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("a character that is not {@a subtype($t)}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            text_phrase(&format!(
                "a character with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            text_phrase(&format!(
                "{} with cost less than the number of allied {}",
                serialize_card_predicate(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            text_phrase(&format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_card_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_card_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_card_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            text_phrase(&format!(
                "{} with cost less than the number of cards in your void",
                serialize_card_predicate(target, bindings)
            ))
        }
    }
}

/// Serialize a card predicate without the article.
///
/// Use this when the caller already provides an article, like "a random".
pub fn serialize_card_predicate_without_article(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            card_predicate_base_phrase(card_predicate)
        }
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("{subtype($t)}")
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            text_phrase(&format!(
                "{} with cost {{energy($e)}}{}",
                serialize_card_predicate_without_article(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_card_predicate_without_article(target, bindings)
            ))
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
) -> Phrase {
    match card_predicate {
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            if is_generic_card_type(target) {
                text_phrase(&format!(
                    "with cost {{energy($e)}}{}",
                    serializer_utils::serialize_operator(cost_operator)
                ))
            } else {
                text_phrase(&format!(
                    "{} with cost {{energy($e)}}{}",
                    serialize_card_predicate_without_article(target, bindings),
                    serializer_utils::serialize_operator(cost_operator)
                ))
            }
        }
        _ => serialize_card_predicate_without_article(card_predicate, bindings),
    }
}

/// Serializes a card predicate in plural form.
pub fn serialize_card_predicate_plural(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            text_phrase(&phrase_plural(&card_predicate_base_phrase(card_predicate)))
        }
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("{@plural subtype($t)}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("characters that are not {@plural subtype($t)}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            text_phrase(&format!(
                "characters with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            text_phrase(&format!(
                "{} with cost less than the number of allied {}",
                serialize_card_predicate_plural(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            text_phrase(&format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_card_predicate_plural(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_card_predicate_plural(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_card_predicate_plural(target, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            text_phrase(&format!(
                "{} with cost less than the number of cards in your void",
                serialize_card_predicate_plural(target, bindings)
            ))
        }
        CardPredicate::Fast { target } => {
            text_phrase(&format!("fast {}", serialize_card_predicate_plural(target, bindings)))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            text_phrase(&format!(
                "{} with cost {{energy($e)}}{}",
                serialize_card_predicate_plural(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            text_phrase("characters with {materialized} abilities")
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            text_phrase("characters with activated abilities")
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_card_predicate_plural(target, bindings)
            ))
        }
        CardPredicate::CouldDissolve { target } => text_phrase(&format!(
            "events which could {{dissolve}} {}",
            predicate_base_text(target, bindings)
        )),
    }
}

/// Serializes the target of a fast card predicate without an article.
pub fn serialize_fast_target(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    match card_predicate {
        CardPredicate::Card => text_phrase("card"),
        CardPredicate::Character => text_phrase("character"),
        CardPredicate::Event => text_phrase("event"),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("{subtype($t)}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("character that is not {@a subtype($t)}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            text_phrase(&format!(
                "character with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            text_phrase(&format!(
                "{} with cost less than the number of allied {}",
                serialize_fast_target(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            text_phrase(&format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_fast_target(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_fast_target(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_fast_target(target, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            text_phrase(&format!(
                "{} with cost less than the number of cards in your void",
                serialize_fast_target(target, bindings)
            ))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            text_phrase(&format!(
                "{} with cost {{energy($e)}}{}",
                serialize_fast_target(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            text_phrase("character with a {materialized} ability")
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            text_phrase("character with an activated ability")
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            text_phrase(&format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_fast_target(target, bindings)
            ))
        }
        CardPredicate::CouldDissolve { target } => text_phrase(&format!(
            "event which could {{dissolve}} {}",
            predicate_base_text(target, bindings)
        )),
        CardPredicate::Fast { target } => {
            text_phrase(&format!("fast {}", serialize_fast_target(target, bindings)))
        }
    }
}

/// Serializes a predicate for "for each" contexts.
pub fn serialize_for_each_predicate(
    predicate: &Predicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    match predicate {
        Predicate::Another(CardPredicate::Character) => text_phrase("allied character"),
        Predicate::Another(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("allied {subtype($t)}")
        }
        Predicate::Your(CardPredicate::Character) => text_phrase("ally"),
        Predicate::Your(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("allied {subtype($t)}")
        }
        Predicate::Enemy(CardPredicate::Character) => text_phrase("enemy"),
        Predicate::Any(CardPredicate::Character) => text_phrase("character"),
        Predicate::Any(CardPredicate::Card) => text_phrase("card"),
        Predicate::YourVoid(CardPredicate::Card) => text_phrase("card in your void"),
        Predicate::This => text_phrase("this character"),
        Predicate::It => text_phrase("that character"),
        Predicate::That => text_phrase("that character"),
        Predicate::Them => text_phrase("character"),
        Predicate::Enemy(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("enemy {subtype($t)}")
        }
        Predicate::Any(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("{subtype($t)}")
        }
        Predicate::Any(CardPredicate::Event) => text_phrase("event"),
        Predicate::AnyOther(CardPredicate::Character) => text_phrase("other character"),
        Predicate::AnyOther(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("other {subtype($t)}")
        }
        Predicate::YourVoid(CardPredicate::Character) => text_phrase("character in your void"),
        Predicate::YourVoid(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            text_phrase("{subtype($t)} in your void")
        }
        Predicate::YourVoid(CardPredicate::Event) => text_phrase("event in your void"),
        Predicate::EnemyVoid(CardPredicate::Card) => text_phrase("card in the opponent's void"),
        Predicate::EnemyVoid(CardPredicate::Character) => {
            text_phrase("character in the opponent's void")
        }
        Predicate::EnemyVoid(CardPredicate::Event) => text_phrase("event in the opponent's void"),
        Predicate::Your(CardPredicate::Event) => text_phrase("allied event"),
        Predicate::Another(CardPredicate::CharacterWithSpark(spark, operator)) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            text_phrase(&format!(
                "ally with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        Predicate::Your(CardPredicate::CharacterWithSpark(spark, operator)) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            text_phrase(&format!(
                "ally with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        predicate => text_phrase(&format!("each {}", predicate_base_text(predicate, bindings))),
    }
}

/// Returns the base noun text for a card predicate without an article.
pub fn card_predicate_base_text(predicate: &CardPredicate) -> String {
    card_predicate_base_phrase(predicate).to_string()
}

/// Returns the plural base noun text for a card predicate.
pub fn card_predicate_base_text_plural(predicate: &CardPredicate) -> String {
    phrase_plural(&card_predicate_base_phrase(predicate))
}

/// Returns the base noun phrase for a card predicate using RLF phrases.
pub fn card_predicate_base_phrase(predicate: &CardPredicate) -> Phrase {
    match predicate {
        CardPredicate::Card => strings::card(),
        CardPredicate::Character => strings::character(),
        CardPredicate::Event => strings::event(),
        CardPredicate::CharacterType(_) => Phrase::builder()
            .text("{subtype($t)}".to_string())
            .variants(
                [(VariantKey::new("other"), "{@plural subtype($t)}".to_string())]
                    .into_iter()
                    .collect(),
            )
            .build(),
        CardPredicate::NotCharacterType(_) => make_phrase("character that is not {@a subtype($t)}"),
        CardPredicate::Fast { target } => card_predicate_base_phrase(target),
        CardPredicate::CardWithCost { target, .. } => card_predicate_base_phrase(target),
        CardPredicate::CharacterWithSpark(..) => strings::character(),
        CardPredicate::CharacterWithMaterializedAbility => strings::character(),
        CardPredicate::CharacterWithMultiActivatedAbility => strings::character(),
        CardPredicate::CouldDissolve { .. } => strings::event(),
        _ => strings::character(),
    }
}

fn your_predicate_formatted(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    match card_predicate {
        CardPredicate::Character => strings::ally(),
        CardPredicate::Card => make_phrase("your card"),
        CardPredicate::Event => make_phrase("your event"),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            make_phrase("allied {subtype($t)}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            make_phrase("ally that is not {@a subtype($t)}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            make_phrase(&format!(
                "ally with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            make_phrase("ally with a {materialized} ability")
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            make_phrase("ally with an activated ability")
        }
        CardPredicate::Fast { target } => {
            make_phrase(&format!("{{fast}} {}", serialize_fast_target(target, bindings)))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            make_phrase(&format!(
                "{} with cost {{energy($e)}}{}",
                serialize_your_predicate(target, bindings),
                serializer_utils::serialize_operator(cost_operator)
            ))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            make_phrase(&format!(
                "{} with cost less than the number of allied {}",
                serialize_your_predicate(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            make_phrase(&format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            make_phrase(&format!(
                "{} with spark less than the abandoned ally's spark",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            make_phrase(&format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            make_phrase(&format!(
                "{} with cost less than the number of cards in your void",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            make_phrase(&format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_your_predicate(target, bindings)
            ))
        }
        CardPredicate::CouldDissolve { target } => make_phrase(&format!(
            "your event which could {{dissolve}} {}",
            predicate_base_text(target, bindings)
        )),
    }
}

fn enemy_predicate_formatted(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> Phrase {
    match card_predicate {
        CardPredicate::Character => strings::enemy(),
        CardPredicate::Card => make_phrase("enemy card"),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            make_phrase("enemy {subtype($t)}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            make_phrase_non_vowel("non-{subtype($t)} enemy")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            make_phrase(&format!(
                "enemy with spark {{$s}}{}",
                serializer_utils::serialize_operator(operator)
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            make_phrase("enemy with a {materialized} ability")
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            make_phrase("enemy with an activated ability")
        }
        CardPredicate::CardWithCost { cost_operator, cost, .. } => {
            bindings.insert("e".to_string(), VariableValue::Integer(cost.0));
            make_phrase(&format!(
                "enemy with cost {{energy($e)}}{}",
                serializer_utils::serialize_operator(cost_operator)
            ))
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            make_phrase(&format!(
                "{} with cost less than the number of allied {}",
                serialize_enemy_predicate(target, bindings),
                serialize_card_predicate_plural(count_matching, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            make_phrase(&format!(
                "{} with cost less than the abandoned ally's cost",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            make_phrase(&format!(
                "{} with cost less than the number of cards in your void",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            make_phrase(&format!(
                "{} with spark less than that ally's spark",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            make_phrase(&format!(
                "{} with spark less than the number of allies abandoned this turn",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            make_phrase(&format!(
                "{} with spark less than the amount of {{energy_symbol}} paid",
                serialize_enemy_predicate(target, bindings)
            ))
        }
        CardPredicate::Fast { target } => {
            make_phrase(&format!("{{fast}} {}", serialize_fast_target(target, bindings)))
        }
        CardPredicate::Event => make_phrase("enemy event"),
        CardPredicate::CouldDissolve { target } => make_phrase(&format!(
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

/// Creates a simple `Phrase` from text with no tags or variants.
fn text_phrase(text: &str) -> Phrase {
    Phrase::builder().text(text.to_string()).build()
}

/// Creates a `Phrase` from a text string, auto-detecting the article tag
/// based on whether the text starts with a vowel sound.
fn make_phrase(text: &str) -> Phrase {
    let tag = if text.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
    Phrase::builder().text(text.to_string()).tags(vec![Tag::new(tag)]).build()
}

/// Creates a `Phrase` with explicit non-vowel article treatment.
fn make_phrase_non_vowel(text: &str) -> Phrase {
    Phrase::builder().text(text.to_string()).tags(vec![Tag::new("a")]).build()
}

/// Returns the text of a phrase with its appropriate article prepended.
fn with_article(phrase: &Phrase) -> String {
    if phrase.has_tag("an") {
        format!("an {}", phrase.text)
    } else {
        format!("a {}", phrase.text)
    }
}

/// Returns the plural form of a phrase by selecting the "other" variant.
///
/// Falls back to appending "s" to the default text if no "other" variant
/// is defined.
fn phrase_plural(phrase: &Phrase) -> String {
    if phrase.variants.contains_key(&VariantKey::new("other")) {
        phrase.variant("other").to_string()
    } else {
        format!("{}s", phrase.text)
    }
}
