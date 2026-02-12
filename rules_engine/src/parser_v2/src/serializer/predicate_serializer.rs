use std::collections::HashMap;

use ability_data::predicate::{CardPredicate, Predicate};
use rlf::{Phrase, VariantKey};
use strings::strings;

use crate::serializer::serializer_utils;

/// Serializes a predicate to a variant-aware Phrase carrying both singular and
/// plural forms. Callers that need plural select the `:other` variant via RLF
/// templates.
pub fn serialize_predicate(predicate: &Predicate) -> Phrase {
    match predicate {
        Predicate::This => strings::this_character(),
        Predicate::That => strings::that_character(),
        Predicate::Them => strings::pronoun_them(),
        Predicate::It => strings::pronoun_it(),
        Predicate::Your(card_predicate) => serialize_your_predicate(card_predicate),
        Predicate::Another(card_predicate) => phrase_with_variants(
            strings::predicate_with_indefinite_article(your_predicate_formatted(card_predicate)),
            serialize_your_predicate_plural(card_predicate),
        ),
        Predicate::Any(card_predicate) => serialize_card_predicate(card_predicate),
        Predicate::Enemy(card_predicate) => phrase_with_variants(
            strings::predicate_with_indefinite_article(enemy_predicate_formatted(card_predicate)),
            serialize_enemy_predicate_plural(card_predicate),
        ),
        Predicate::YourVoid(card_predicate) => {
            strings::in_your_void(serialize_card_predicate(card_predicate))
        }
        Predicate::EnemyVoid(card_predicate) => {
            strings::in_opponent_void(serialize_card_predicate(card_predicate))
        }
        Predicate::AnyOther(card_predicate) => {
            strings::another_pred(card_predicate_base_phrase(card_predicate))
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
                strings::with_cost_constraint(
                    serializer_utils::serialize_operator(cost_operator),
                    cost.0,
                )
            } else {
                strings::pred_with_constraint(
                    serialize_card_predicate_without_article(target),
                    strings::with_cost_constraint(
                        serializer_utils::serialize_operator(cost_operator),
                        cost.0,
                    ),
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
            strings::in_your_void(serialize_card_predicate(card_predicate))
        }
        Predicate::EnemyVoid(card_predicate) => {
            strings::in_opponent_void(serialize_card_predicate(card_predicate))
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
    card_predicate_base_phrase(predicate).to_string()
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
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            strings::pred_with_constraint(
                serialize_card_predicate(target),
                strings::with_cost_constraint(
                    serializer_utils::serialize_operator(cost_operator),
                    cost.0,
                ),
            )
        }
        CardPredicate::CharacterWithMaterializedAbility => strings::pred_with_constraint(
            serialize_card_predicate(&CardPredicate::Character),
            strings::with_materialized_ability_constraint(),
        ),
        CardPredicate::CharacterWithMultiActivatedAbility => strings::pred_with_constraint(
            serialize_card_predicate(&CardPredicate::Character),
            strings::with_activated_ability_constraint(),
        ),
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            strings::pred_with_constraint(
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
        CardPredicate::CharacterWithSpark(spark, operator) => strings::pred_with_constraint(
            serialize_card_predicate(&CardPredicate::Character),
            strings::with_spark_constraint(serializer_utils::serialize_operator(operator), spark.0),
        ),
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            strings::pred_with_constraint(
                serialize_card_predicate(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate(count_matching)),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                serialize_card_predicate(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                serialize_card_predicate(target),
                strings::with_spark_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            strings::pred_with_constraint(
                serialize_card_predicate(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            strings::pred_with_constraint(
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
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            strings::pred_with_constraint(
                serialize_card_predicate_without_article(target),
                strings::with_cost_constraint(
                    serializer_utils::serialize_operator(cost_operator),
                    cost.0,
                ),
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            strings::pred_with_constraint(
                serialize_card_predicate_without_article(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
        }
        _ => serialize_card_predicate(card_predicate),
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

/// Serializes a Your predicate, returning a variant-aware phrase.
///
/// For simple types (Character, Card, Event, CharacterType), the phrase is
/// natively variant-aware via your_generic_* helpers. For compound predicates,
/// the singular uses the generic `serialize_card_predicate` (matching Any
/// behavior) and the plural uses `serialize_your_predicate_plural` (adding
/// "ally"/"allied" ownership). These are packaged via `with_plural`.
fn serialize_your_predicate(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Character => {
            strings::predicate_with_indefinite_article(strings::your_generic_character())
        }
        CardPredicate::Card => {
            strings::predicate_with_indefinite_article(strings::your_generic_card())
        }
        CardPredicate::Event => {
            strings::predicate_with_indefinite_article(strings::your_generic_event())
        }
        CardPredicate::CharacterType(subtype) => strings::predicate_with_indefinite_article(
            strings::your_generic_subtype(serializer_utils::subtype_to_phrase(*subtype)),
        ),
        CardPredicate::CouldDissolve { target } => strings::predicate_with_indefinite_article(
            strings::could_dissolve_target(serialize_predicate(target)),
        ),
        _ => strings::with_plural(
            serialize_card_predicate(card_predicate),
            serialize_your_predicate_plural(card_predicate),
        ),
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
        CardPredicate::CharacterWithSpark(spark, operator) => strings::pred_with_constraint(
            strings::ally(),
            strings::with_spark_constraint(serializer_utils::serialize_operator(operator), spark.0),
        ),
        CardPredicate::CharacterWithMaterializedAbility => strings::pred_with_constraint(
            strings::ally(),
            strings::with_materialized_ability_constraint(),
        ),
        CardPredicate::CharacterWithMultiActivatedAbility => strings::pred_with_constraint(
            strings::ally(),
            strings::with_activated_ability_constraint(),
        ),
        CardPredicate::Fast { target } => strings::fast_predicate(serialize_fast_target(target)),
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            strings::pred_with_constraint(
                your_predicate_formatted(target),
                strings::with_cost_constraint(
                    serializer_utils::serialize_operator(cost_operator),
                    cost.0,
                ),
            )
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            strings::pred_with_constraint(
                your_predicate_formatted(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate(count_matching)),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                your_predicate_formatted(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                your_predicate_formatted(target),
                strings::with_spark_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            strings::pred_with_constraint(
                your_predicate_formatted(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            strings::pred_with_constraint(
                your_predicate_formatted(target),
                strings::with_cost_less_than_void_count_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            strings::pred_with_constraint(
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
        CardPredicate::CharacterWithSpark(spark, operator) => strings::pred_with_constraint(
            strings::enemy(),
            strings::with_spark_constraint(serializer_utils::serialize_operator(operator), spark.0),
        ),
        CardPredicate::CharacterWithMaterializedAbility => strings::pred_with_constraint(
            strings::enemy(),
            strings::with_materialized_ability_constraint(),
        ),
        CardPredicate::CharacterWithMultiActivatedAbility => strings::pred_with_constraint(
            strings::enemy(),
            strings::with_activated_ability_constraint(),
        ),
        CardPredicate::CardWithCost { cost_operator, cost, .. } => strings::pred_with_constraint(
            strings::enemy(),
            strings::with_cost_constraint(
                serializer_utils::serialize_operator(cost_operator),
                cost.0,
            ),
        ),
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            strings::pred_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate(count_matching)),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            strings::pred_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_cost_less_than_void_count_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_spark_less_than_that_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            strings::pred_with_constraint(
                enemy_predicate_formatted(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            strings::pred_with_constraint(
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

/// Returns a phrase whose default text is the plural form of a "your"
/// predicate. Used as the `$other` argument to `with_plural`.
fn serialize_your_predicate_plural(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Character => strings::as_plural(strings::ally()),
        CardPredicate::Card => strings::as_plural(strings::your_card()),
        CardPredicate::Event => strings::as_plural(strings::your_event()),
        CardPredicate::CharacterType(subtype) => strings::as_plural(
            strings::allied_card_with_subtype(serializer_utils::subtype_to_phrase(*subtype)),
        ),
        CardPredicate::Fast { target } => {
            strings::fast_predicate_plural(serialize_card_predicate_plural_text(target))
        }
        CardPredicate::NotCharacterType(subtype) => {
            strings::ally_not_subtype_plural(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            strings::as_plural(strings::pred_with_constraint(
                strings::ally(),
                strings::with_spark_constraint(
                    serializer_utils::serialize_operator(operator),
                    spark.0,
                ),
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            strings::as_plural(strings::pred_with_constraint(
                strings::ally(),
                strings::with_materialized_abilities_constraint(),
            ))
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            strings::as_plural(strings::pred_with_constraint(
                strings::ally(),
                strings::with_activated_abilities_constraint(),
            ))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            strings::pred_with_constraint(
                serialize_your_predicate_plural(target),
                strings::with_cost_constraint(
                    serializer_utils::serialize_operator(cost_operator),
                    cost.0,
                ),
            )
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            strings::pred_with_constraint(
                serialize_your_predicate_plural(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate(count_matching)),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                serialize_your_predicate_plural(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                serialize_your_predicate_plural(target),
                strings::with_spark_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            strings::pred_with_constraint(
                serialize_your_predicate_plural(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            strings::pred_with_constraint(
                serialize_your_predicate_plural(target),
                strings::with_cost_less_than_void_count_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            strings::pred_with_constraint(
                serialize_your_predicate_plural(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
        }
        CardPredicate::CouldDissolve { target } => {
            strings::your_event_could_dissolve_plural(serialize_predicate(target))
        }
    }
}

/// Returns a phrase whose default text is the plural form of an "enemy"
/// predicate.
fn serialize_enemy_predicate_plural(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Character => strings::as_plural(strings::enemy()),
        CardPredicate::Card => strings::as_plural(strings::enemy_card()),
        CardPredicate::Event => strings::as_plural(strings::enemy_event()),
        CardPredicate::CharacterType(subtype) => {
            strings::enemy_subtype_plural(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::NotCharacterType(subtype) => {
            strings::non_subtype_enemy_plural(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::Fast { target } => {
            strings::fast_predicate_plural(serialize_card_predicate_plural_text(target))
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            strings::as_plural(strings::pred_with_constraint(
                strings::enemy(),
                strings::with_spark_constraint(
                    serializer_utils::serialize_operator(operator),
                    spark.0,
                ),
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            strings::as_plural(strings::pred_with_constraint(
                strings::enemy(),
                strings::with_materialized_abilities_constraint(),
            ))
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            strings::as_plural(strings::pred_with_constraint(
                strings::enemy(),
                strings::with_activated_abilities_constraint(),
            ))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            strings::pred_with_constraint(
                serialize_enemy_predicate_plural(target),
                strings::with_cost_constraint(
                    serializer_utils::serialize_operator(cost_operator),
                    cost.0,
                ),
            )
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            strings::pred_with_constraint(
                serialize_enemy_predicate_plural(target),
                strings::with_cost_less_than_allied_count(serialize_card_predicate(count_matching)),
            )
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                serialize_enemy_predicate_plural(target),
                strings::with_cost_less_than_abandoned_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            strings::pred_with_constraint(
                serialize_enemy_predicate_plural(target),
                strings::with_spark_less_than_that_ally_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            strings::pred_with_constraint(
                serialize_enemy_predicate_plural(target),
                strings::with_spark_less_than_abandoned_count_this_turn_constraint(),
            )
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            strings::pred_with_constraint(
                serialize_enemy_predicate_plural(target),
                strings::with_cost_less_than_void_count_constraint(),
            )
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            strings::pred_with_constraint(
                serialize_enemy_predicate_plural(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            )
        }
        CardPredicate::CouldDissolve { target } => {
            strings::could_dissolve_target_plural(serialize_predicate(target))
        }
    }
}

/// Returns a phrase whose default text is the plural form of a card predicate
/// (without ownership qualifiers).
fn serialize_card_predicate_plural_text(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            strings::as_plural(card_predicate_base_phrase(card_predicate))
        }
        CardPredicate::CharacterType(subtype) => {
            strings::as_plural(strings::subtype(serializer_utils::subtype_to_phrase(*subtype)))
        }
        CardPredicate::NotCharacterType(subtype) => {
            strings::character_not_subtype_plural(serializer_utils::subtype_to_phrase(*subtype))
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            strings::as_plural(strings::pred_with_constraint(
                strings::character(),
                strings::with_spark_constraint(
                    serializer_utils::serialize_operator(operator),
                    spark.0,
                ),
            ))
        }
        CardPredicate::Fast { target } => {
            strings::fast_predicate_plural(serialize_card_predicate_plural_text(target))
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            strings::as_plural(strings::pred_with_constraint(
                serialize_card_predicate(target),
                strings::with_cost_constraint(
                    serializer_utils::serialize_operator(cost_operator),
                    cost.0,
                ),
            ))
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            strings::as_plural(strings::pred_with_constraint(
                strings::character(),
                strings::with_materialized_abilities_constraint(),
            ))
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            strings::as_plural(strings::pred_with_constraint(
                strings::character(),
                strings::with_activated_abilities_constraint(),
            ))
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            strings::as_plural(strings::pred_with_constraint(
                serialize_card_predicate(target),
                strings::with_spark_less_than_energy_paid_constraint(),
            ))
        }
        CardPredicate::CouldDissolve { target } => {
            strings::could_dissolve_target_plural(serialize_predicate(target))
        }
        _ => strings::as_plural(card_predicate_base_phrase(card_predicate)),
    }
}

/// Returns true if the card predicate is a generic type (Card, Character,
/// Event) without any modifiers. These are serialized without ownership
/// qualifiers for round-trip compatibility.
fn is_generic_card_type(card_predicate: &CardPredicate) -> bool {
    matches!(card_predicate, CardPredicate::Card | CardPredicate::Character | CardPredicate::Event)
}

/// Builds a variant-aware Phrase from separate singular and plural RLF
/// phrases.
///
/// This is used when the RLF macro cannot create variant blocks for
/// parameterized phrases without `:from`. It constructs a Phrase with `one`
/// and `other` variant keys, inheriting the tag from the singular phrase.
fn phrase_with_variants(singular: Phrase, plural: Phrase) -> Phrase {
    Phrase::builder()
        .text(singular.text.clone())
        .variants(HashMap::from([
            (VariantKey::new("one"), singular.text),
            (VariantKey::new("other"), plural.text),
        ]))
        .tags(singular.tags)
        .build()
}
