use ability_data::predicate::{CardPredicate, Operator, Predicate};
use core_data::numerics::Energy;
use rlf::Phrase;
use strings::strings;

use crate::serializer::serializer_utils;

/// Serializes a predicate to a Phrase.
pub fn serialize_predicate(predicate: &Predicate) -> Phrase {
    match predicate {
        Predicate::This => strings::this_character(),
        Predicate::That => strings::that_character(),
        Predicate::Them => strings::pronoun_them(),
        Predicate::It => strings::pronoun_it(),
        Predicate::Your(card_predicate) => {
            if let CardPredicate::CharacterType(subtype) = card_predicate {
                strings::predicate_with_indefinite_article(strings::subtype(
                    serializer_utils::subtype_to_phrase(*subtype),
                ))
            } else {
                serialize_card_predicate(card_predicate)
            }
        }
        Predicate::Another(card_predicate) => {
            strings::predicate_with_indefinite_article(your_predicate_formatted(card_predicate))
        }
        Predicate::Any(card_predicate) => serialize_card_predicate(card_predicate),
        Predicate::Enemy(card_predicate) => {
            strings::predicate_with_indefinite_article(enemy_predicate_formatted(card_predicate))
        }
        Predicate::YourVoid(card_predicate) => {
            strings::in_your_void(serialize_card_predicate_plural(card_predicate))
        }
        Predicate::EnemyVoid(card_predicate) => {
            strings::in_opponent_void(serialize_card_predicate_plural(card_predicate))
        }
        Predicate::AnyOther(card_predicate) => {
            strings::another_pred(card_predicate_base_phrase(card_predicate))
        }
    }
}

/// Serializes a predicate in plural form.
pub fn serialize_predicate_plural(predicate: &Predicate) -> Phrase {
    match predicate {
        Predicate::Another(card_predicate) => {
            serialize_your_predicate_plural_phrase(card_predicate)
        }
        Predicate::Any(card_predicate) => serialize_card_predicate_plural(card_predicate),
        Predicate::Your(card_predicate) => serialize_your_predicate_plural_phrase(card_predicate),
        Predicate::Enemy(card_predicate) => serialize_enemy_predicate_plural_phrase(card_predicate),
        Predicate::YourVoid(card_predicate) => {
            strings::in_your_void(serialize_card_predicate_plural(card_predicate))
        }
        Predicate::EnemyVoid(card_predicate) => {
            strings::in_opponent_void(serialize_card_predicate_plural(card_predicate))
        }
        Predicate::This => strings::these_characters(),
        Predicate::That => strings::those_characters(),
        Predicate::Them | Predicate::It => strings::pronoun_them(),
        Predicate::AnyOther(card_predicate) => {
            strings::other_pred_plural(card_predicate_base_phrase(card_predicate))
        }
    }
}

/// Serialize only the cost constraint part of a card predicate.
///
/// This is used when the base card type is already implied by context (e.g.,
/// `{n_random_characters(number)}` already says "characters", so we only need
/// "with cost {energy(e)} or less").
pub fn serialize_cost_constraint_only(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            if is_generic_card_type(target) {
                serialize_cost_constraint(cost_operator, *cost)
            } else {
                compose_with_constraint(
                    serialize_card_predicate_without_article(target),
                    serialize_cost_constraint(cost_operator, *cost),
                )
            }
        }
        _ => serialize_card_predicate_without_article(card_predicate),
    }
}

/// Serializes a predicate to its base phrase without an article.
pub(super) fn predicate_base_phrase(predicate: &Predicate) -> Phrase {
    match predicate {
        Predicate::This => strings::this_character(),
        Predicate::That => strings::that_character(),
        Predicate::Them => strings::pronoun_them(),
        Predicate::It => strings::pronoun_it(),
        Predicate::Your(card_predicate) => {
            if is_generic_card_type(card_predicate) {
                card_predicate_base_phrase(card_predicate)
            } else {
                your_predicate_formatted(card_predicate)
            }
        }
        Predicate::Another(card_predicate) => your_predicate_formatted(card_predicate),
        Predicate::Any(card_predicate) => card_predicate_base_phrase(card_predicate),
        Predicate::Enemy(card_predicate) => match card_predicate {
            _ if is_generic_card_type(card_predicate) => card_predicate_base_phrase(card_predicate),
            CardPredicate::CouldDissolve { target } => {
                strings::could_dissolve_target(serialize_predicate(target))
            }
            CardPredicate::CardWithCost { .. } => {
                serialize_card_predicate_without_article(card_predicate)
            }
            _ => enemy_predicate_formatted(card_predicate),
        },
        Predicate::YourVoid(card_predicate) => {
            strings::in_your_void(serialize_card_predicate_plural(card_predicate))
        }
        Predicate::EnemyVoid(card_predicate) => {
            strings::in_opponent_void(serialize_card_predicate_plural(card_predicate))
        }
        Predicate::AnyOther(card_predicate) => {
            strings::another_pred(card_predicate_base_phrase(card_predicate))
        }
    }
}

/// Serializes a card predicate with an article.
pub(super) fn serialize_card_predicate_phrase(card_predicate: &CardPredicate) -> Phrase {
    serialize_card_predicate(card_predicate)
}

/// Serializes a card predicate in plural form.
pub(super) fn serialize_card_predicate_plural_phrase(card_predicate: &CardPredicate) -> Phrase {
    serialize_card_predicate_plural(card_predicate)
}

/// Serializes a card predicate without an article.
pub(super) fn card_predicate_without_article(card_predicate: &CardPredicate) -> Phrase {
    serialize_card_predicate_without_article(card_predicate)
}

/// Returns the base noun phrase for a card predicate using RLF phrases.
pub(super) fn base_card_phrase(predicate: &CardPredicate) -> Phrase {
    card_predicate_base_phrase(predicate)
}

/// Returns the base noun text for a card predicate without an article.
pub(super) fn base_card_text(predicate: &CardPredicate) -> String {
    card_predicate_base_text(predicate)
}

/// Returns the plural base noun text for a card predicate.
pub(super) fn base_card_text_plural(predicate: &CardPredicate) -> String {
    card_predicate_base_text_plural(predicate)
}

/// Serializes a predicate for "for each" contexts.
pub(super) fn for_each_predicate_phrase(predicate: &Predicate) -> Phrase {
    serialize_for_each_predicate(predicate)
}

/// Serializes a card predicate with an article.
fn serialize_card_predicate(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            strings::predicate_with_indefinite_article(card_predicate_base_phrase(card_predicate))
        }
        CardPredicate::CharacterType(subtype) => strings::predicate_with_indefinite_article(
            strings::subtype(serializer_utils::subtype_to_phrase(*subtype)),
        ),
        CardPredicate::Fast { target } => strings::predicate_with_indefinite_article(
            strings::fast_predicate(serialize_fast_target(target)),
        ),
        CardPredicate::CardWithCost { target, cost_operator, cost } => compose_with_constraint(
            serialize_card_predicate(target),
            serialize_cost_constraint(cost_operator, *cost),
        ),
        CardPredicate::CharacterWithMaterializedAbility => compose_with_constraint(
            serialize_card_predicate(&CardPredicate::Character),
            strings::with_materialized_ability_constraint(),
        ),
        CardPredicate::CharacterWithMultiActivatedAbility => compose_with_constraint(
            serialize_card_predicate(&CardPredicate::Character),
            strings::with_activated_ability_constraint(),
        ),
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
        }
        CardPredicate::CouldDissolve { target } => strings::predicate_with_indefinite_article(
            strings::could_dissolve_target(serialize_predicate(target)),
        ),
        CardPredicate::NotCharacterType(subtype) => strings::predicate_with_indefinite_article(
            strings::character_not_subtype(serializer_utils::subtype_to_phrase(*subtype)),
        ),
        CardPredicate::CharacterWithSpark(spark, operator) => compose_with_constraint(
            serialize_card_predicate(&CardPredicate::Character),
            strings::with_spark_constraint(serializer_utils::serialize_operator(operator), spark.0),
        ),
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            compose_with_constraint(
                serialize_card_predicate(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate_plural(
                    count_matching,
                )),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate(target),
                strings::with_spark_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate(target),
                strings::with_cost_less_than_void_count_constraint(),
            )
        }
    }
}

/// Serialize a card predicate without the article.
///
/// Use this when the caller already provides an article, like "a random".
fn serialize_card_predicate_without_article(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            card_predicate_base_phrase(card_predicate)
        }
        CardPredicate::CharacterType(subtype) => {
            strings::subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => compose_with_constraint(
            serialize_card_predicate_without_article(target),
            serialize_cost_constraint(cost_operator, *cost),
        ),
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate_without_article(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
        }
        _ => serialize_card_predicate(card_predicate),
    }
}

/// Serializes a card predicate in plural form.
fn serialize_card_predicate_plural(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            plural_phrase(&card_predicate_base_phrase(card_predicate))
        }
        CardPredicate::CharacterType(subtype) => {
            plural_phrase(&strings::subtype(serializer_utils::subtype_to_phrase(*subtype)))
        }
        CardPredicate::NotCharacterType(subtype) => {
            strings::characters_not_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::CharacterWithSpark(spark, operator) => compose_with_constraint(
            plural_phrase(&strings::character()),
            strings::with_spark_constraint(serializer_utils::serialize_operator(operator), spark.0),
        ),
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            compose_with_constraint(
                serialize_card_predicate_plural(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate_plural(
                    count_matching,
                )),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate_plural(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate_plural(target),
                strings::with_spark_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate_plural(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate_plural(target),
                strings::with_cost_less_than_void_count_constraint(),
            )
        }
        CardPredicate::Fast { target } => {
            strings::fast_predicate(serialize_card_predicate_plural(target))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => compose_with_constraint(
            serialize_card_predicate_plural(target),
            serialize_cost_constraint(cost_operator, *cost),
        ),
        CardPredicate::CharacterWithMaterializedAbility => compose_with_constraint(
            plural_phrase(&strings::character()),
            strings::with_materialized_abilities_constraint(),
        ),
        CardPredicate::CharacterWithMultiActivatedAbility => compose_with_constraint(
            plural_phrase(&strings::character()),
            strings::with_activated_abilities_constraint(),
        ),
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            compose_with_constraint(
                serialize_card_predicate_plural(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
        }
        CardPredicate::CouldDissolve { target } => {
            strings::events_could_dissolve_target(predicate_base_phrase(target))
        }
    }
}

fn serialize_fast_target(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Fast { target } => strings::fast_predicate(serialize_fast_target(target)),
        _ => serialize_card_predicate_without_article(card_predicate),
    }
}

fn serialize_for_each_predicate(predicate: &Predicate) -> Phrase {
    match predicate {
        Predicate::Another(CardPredicate::Character) => strings::for_each_allied_character(),
        Predicate::Another(CardPredicate::CharacterType(subtype)) => {
            strings::for_each_allied_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        Predicate::Your(CardPredicate::Character) => strings::for_each_ally(),
        Predicate::Your(CardPredicate::CharacterType(subtype)) => {
            strings::for_each_allied_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        Predicate::Enemy(CardPredicate::Character) => strings::for_each_enemy(),
        Predicate::Any(CardPredicate::Character) => strings::for_each_character(),
        Predicate::Any(CardPredicate::Card) => strings::for_each_card(),
        Predicate::YourVoid(CardPredicate::Card) => strings::for_each_card_in_your_void(),
        Predicate::This => strings::for_each_this_character(),
        Predicate::It => strings::for_each_that_character(),
        Predicate::That => strings::for_each_that_character(),
        Predicate::Them => strings::for_each_character(),
        Predicate::Enemy(CardPredicate::CharacterType(subtype)) => {
            strings::for_each_enemy_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        Predicate::Any(CardPredicate::CharacterType(subtype)) => {
            strings::for_each_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        Predicate::Any(CardPredicate::Event) => strings::for_each_event(),
        Predicate::AnyOther(CardPredicate::Character) => strings::for_each_other_character(),
        Predicate::AnyOther(CardPredicate::CharacterType(subtype)) => {
            strings::for_each_other_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        Predicate::YourVoid(CardPredicate::Character) => strings::for_each_character_in_your_void(),
        Predicate::YourVoid(CardPredicate::CharacterType(subtype)) => {
            strings::for_each_subtype_in_your_void(serializer_utils::subtype_to_phrase(*subtype))
        }
        Predicate::YourVoid(CardPredicate::Event) => strings::for_each_event_in_your_void(),
        Predicate::EnemyVoid(CardPredicate::Card) => strings::for_each_card_in_enemy_void(),
        Predicate::EnemyVoid(CardPredicate::Character) => {
            strings::for_each_character_in_enemy_void()
        }
        Predicate::EnemyVoid(CardPredicate::Event) => strings::for_each_event_in_enemy_void(),
        Predicate::Your(CardPredicate::Event) => strings::for_each_allied_event(),
        Predicate::Another(CardPredicate::CharacterWithSpark(spark, operator))
        | Predicate::Your(CardPredicate::CharacterWithSpark(spark, operator)) => {
            strings::for_each_ally_with_spark(
                spark.0,
                serializer_utils::serialize_operator(operator),
            )
        }
        predicate => strings::for_each_predicate(predicate_base_phrase(predicate)),
    }
}

fn card_predicate_base_text(predicate: &CardPredicate) -> String {
    card_predicate_base_phrase(predicate).to_string()
}

fn card_predicate_base_text_plural(predicate: &CardPredicate) -> String {
    plural_phrase(&card_predicate_base_phrase(predicate)).to_string()
}

fn card_predicate_base_phrase(predicate: &CardPredicate) -> Phrase {
    match predicate {
        CardPredicate::Card => strings::card(),
        CardPredicate::Character => strings::character(),
        CardPredicate::Event => strings::event(),
        CardPredicate::CharacterType(subtype) => {
            strings::subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::NotCharacterType(subtype) => {
            strings::character_not_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::Fast { target } => card_predicate_base_phrase(target),
        CardPredicate::CardWithCost { target, .. } => card_predicate_base_phrase(target),
        CardPredicate::CharacterWithSpark(..) => strings::character(),
        CardPredicate::CharacterWithMaterializedAbility => strings::character(),
        CardPredicate::CharacterWithMultiActivatedAbility => strings::character(),
        CardPredicate::CouldDissolve { .. } => strings::event(),
        _ => strings::character(),
    }
}

fn serialize_your_predicate_plural_phrase(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Character => {
            Phrase::builder().text(strings::ally().variant("other").to_string()).build()
        }
        CardPredicate::Card => {
            Phrase::builder().text(strings::your_card().variant("other").to_string()).build()
        }
        CardPredicate::Event => {
            Phrase::builder().text(strings::your_event().variant("other").to_string()).build()
        }
        CardPredicate::CharacterType(subtype) => {
            strings::allied_subtype_plural(serializer_utils::subtype_to_phrase(*subtype))
        }
        _ => Phrase::builder().text(serialize_your_predicate_plural(card_predicate)).build(),
    }
}

fn serialize_enemy_predicate_plural_phrase(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Character => {
            Phrase::builder().text(strings::enemy().variant("other").to_string()).build()
        }
        CardPredicate::Card => {
            Phrase::builder().text(strings::enemy_card().variant("other").to_string()).build()
        }
        CardPredicate::Event => {
            Phrase::builder().text(strings::enemy_event().variant("other").to_string()).build()
        }
        CardPredicate::CharacterType(subtype) => {
            strings::enemy_subtype_plural(serializer_utils::subtype_to_phrase(*subtype))
        }
        _ => Phrase::builder().text(serialize_enemy_predicate_plural(card_predicate)).build(),
    }
}

fn your_predicate_formatted(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Character => strings::ally(),
        CardPredicate::Card => strings::your_card(),
        CardPredicate::Event => strings::your_event(),
        CardPredicate::CharacterType(subtype) => {
            strings::allied_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::NotCharacterType(subtype) => {
            strings::ally_not_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::CharacterWithSpark(spark, operator) => compose_with_constraint(
            strings::ally(),
            strings::with_spark_constraint(serializer_utils::serialize_operator(operator), spark.0),
        ),
        CardPredicate::CharacterWithMaterializedAbility => compose_with_constraint(
            strings::ally(),
            strings::with_materialized_ability_constraint(),
        ),
        CardPredicate::CharacterWithMultiActivatedAbility => {
            compose_with_constraint(strings::ally(), strings::with_activated_ability_constraint())
        }
        CardPredicate::Fast { target } => strings::fast_predicate(serialize_fast_target(target)),
        CardPredicate::CardWithCost { target, cost_operator, cost } => compose_with_constraint(
            your_predicate_formatted(target),
            serialize_cost_constraint(cost_operator, *cost),
        ),
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            compose_with_constraint(
                your_predicate_formatted(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate_plural(
                    count_matching,
                )),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                your_predicate_formatted(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                your_predicate_formatted(target),
                strings::with_spark_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            compose_with_constraint(
                your_predicate_formatted(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            compose_with_constraint(
                your_predicate_formatted(target),
                strings::with_cost_less_than_void_count_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            compose_with_constraint(
                your_predicate_formatted(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
        }
        CardPredicate::CouldDissolve { target } => {
            strings::your_event_could_dissolve(serialize_predicate(target))
        }
    }
}

fn enemy_predicate_formatted(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Character => strings::enemy(),
        CardPredicate::Card => strings::enemy_card(),
        CardPredicate::CharacterType(subtype) => {
            strings::enemy_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::NotCharacterType(subtype) => {
            strings::non_subtype_enemy(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::CharacterWithSpark(spark, operator) => compose_with_constraint(
            strings::enemy(),
            strings::with_spark_constraint(serializer_utils::serialize_operator(operator), spark.0),
        ),
        CardPredicate::CharacterWithMaterializedAbility => compose_with_constraint(
            strings::enemy(),
            strings::with_materialized_ability_constraint(),
        ),
        CardPredicate::CharacterWithMultiActivatedAbility => {
            compose_with_constraint(strings::enemy(), strings::with_activated_ability_constraint())
        }
        CardPredicate::CardWithCost { cost_operator, cost, .. } => compose_with_constraint(
            strings::enemy(),
            serialize_cost_constraint(cost_operator, *cost),
        ),
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            compose_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate_plural(
                    count_matching,
                )),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            compose_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_cost_less_than_void_count_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_spark_less_than_that_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            compose_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            compose_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
        }
        CardPredicate::Fast { target } => strings::fast_predicate(serialize_fast_target(target)),
        CardPredicate::Event => strings::enemy_event(),
        CardPredicate::CouldDissolve { target } => {
            strings::could_dissolve_target(serialize_predicate(target))
        }
    }
}

fn serialize_your_predicate_plural(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => strings::ally().variant("other").to_string(),
        CardPredicate::Card => strings::your_card().variant("other").to_string(),
        CardPredicate::Event => strings::your_event().variant("other").to_string(),
        CardPredicate::CharacterType(subtype) => {
            strings::allied_subtype_plural(serializer_utils::subtype_to_phrase(*subtype))
                .to_string()
        }
        CardPredicate::Fast { target } => {
            strings::fast_predicate(serialize_card_predicate_plural(target)).to_string()
        }
        CardPredicate::NotCharacterType(subtype) => {
            strings::allies_not_subtype(serializer_utils::subtype_to_phrase(*subtype)).to_string()
        }
        CardPredicate::CharacterWithSpark(spark, operator) => compose_with_constraint(
            plural_phrase(&strings::ally()),
            strings::with_spark_constraint(serializer_utils::serialize_operator(operator), spark.0),
        )
        .to_string(),
        CardPredicate::CharacterWithMaterializedAbility => compose_with_constraint(
            plural_phrase(&strings::ally()),
            strings::with_materialized_abilities_constraint(),
        )
        .to_string(),
        CardPredicate::CharacterWithMultiActivatedAbility => compose_with_constraint(
            plural_phrase(&strings::ally()),
            strings::with_activated_abilities_constraint(),
        )
        .to_string(),
        CardPredicate::CardWithCost { target, cost_operator, cost } => compose_with_constraint(
            Phrase::builder().text(serialize_your_predicate_plural(target)).build(),
            serialize_cost_constraint(cost_operator, *cost),
        )
        .to_string(),
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            compose_with_constraint(
                Phrase::builder().text(serialize_your_predicate_plural(target)).build(),
                strings::with_cost_less_than_allied_count(serialize_card_predicate_plural(
                    count_matching,
                )),
            )
            .to_string()
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                Phrase::builder().text(serialize_your_predicate_plural(target)).build(),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
            .to_string()
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            compose_with_constraint(
                Phrase::builder().text(serialize_your_predicate_plural(target)).build(),
                strings::with_spark_less_than_abandoned_ally_constraint(),
            )
            .to_string()
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            compose_with_constraint(
                Phrase::builder().text(serialize_your_predicate_plural(target)).build(),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
            .to_string()
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            compose_with_constraint(
                Phrase::builder().text(serialize_your_predicate_plural(target)).build(),
                strings::with_cost_less_than_void_count_constraint(),
            )
            .to_string()
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            compose_with_constraint(
                Phrase::builder().text(serialize_your_predicate_plural(target)).build(),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
            .to_string()
        }
        CardPredicate::CouldDissolve { target } => {
            strings::your_events_could_dissolve(serialize_predicate(target)).to_string()
        }
    }
}

fn serialize_enemy_predicate_plural(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => strings::enemy().variant("other").to_string(),
        CardPredicate::CharacterType(subtype) => {
            strings::enemy_subtype_plural(serializer_utils::subtype_to_phrase(*subtype)).to_string()
        }
        _ => strings::enemy_pred(serialize_card_predicate_plural(card_predicate)).to_string(),
    }
}

/// Returns true if the card predicate is a generic type (Card, Character,
/// Event) without any modifiers. These are serialized without ownership
/// qualifiers for round-trip compatibility.
fn is_generic_card_type(card_predicate: &CardPredicate) -> bool {
    matches!(card_predicate, CardPredicate::Card | CardPredicate::Character | CardPredicate::Event)
}

fn compose_with_constraint(base: Phrase, constraint: Phrase) -> Phrase {
    strings::pred_with_constraint(base, constraint)
}

fn plural_phrase(phrase: &Phrase) -> Phrase {
    Phrase::builder().text(phrase.variant("other").to_string()).build()
}

fn serialize_cost_constraint(cost_operator: &Operator<Energy>, cost: Energy) -> Phrase {
    strings::with_cost_constraint(serializer_utils::serialize_operator(cost_operator), cost.0)
}
