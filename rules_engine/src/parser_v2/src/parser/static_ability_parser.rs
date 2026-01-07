use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::{
    AlternateCost, PlayFromHandOrVoidForCost, StandardStaticAbility, StaticAbility,
    StaticAbilityWithOptions,
};
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser::parser_helpers::{
    article, colon, comma, directive, energy, period, spark, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, condition_parser, cost_parser, predicate_parser};

/// Parses static abilities that apply continuously.
pub fn static_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StaticAbility, ParserExtra<'a>> + Clone {
    choice((
        condition_parser::condition_parser()
            .then(standard_static_ability())
            .map(|(condition, ability)| {
                StaticAbility::WithOptions(StaticAbilityWithOptions {
                    ability,
                    condition: Some(condition),
                })
            })
            .boxed(),
        standard_static_ability_without_period()
            .then_ignore(word("if"))
            .then(condition_parser::condition_parser())
            .then_ignore(period())
            .map(|(ability, condition)| {
                StaticAbility::WithOptions(StaticAbilityWithOptions {
                    ability,
                    condition: Some(condition),
                })
            })
            .boxed(),
        standard_static_ability().map(StaticAbility::StaticAbility),
    ))
    .boxed()
}

fn standard_static_ability<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    choice((
        play_only_from_void(),
        cards_in_your_void_have_reclaim(),
        additional_cost_to_play(),
        once_per_turn_play_from_void(),
        abandon_ally_play_character_for_alternate_cost(),
        play_for_alternate_cost(),
        play_from_hand_or_void_for_cost(),
        simple_alternate_cost().then_ignore(period()),
        characters_in_hand_have_fast(),
        disable_enemy_materialized_abilities(),
        has_all_character_types(),
        allied_spark_bonus(),
        spark_equal_to_predicate_count(),
        enemy_cards_cost_increase(),
        your_cards_cost_modification(),
        reveal_top_card_of_deck(),
        play_from_top_of_deck(),
        judgment_triggers_when_materialized(),
    ))
    .boxed()
}

fn standard_static_ability_without_period<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    simple_alternate_cost().boxed()
}

fn your_cards_cost_modification<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    card_predicate_parser::parser()
        .then_ignore(words(&["cost", "you"]))
        .then(energy())
        .then(choice((word("less").to(false), word("more").to(true))))
        .then_ignore(period())
        .map(|((matching, amount), is_increase)| {
            if is_increase {
                StandardStaticAbility::YourCardsCostIncrease { matching, reduction: Energy(amount) }
            } else {
                StandardStaticAbility::YourCardsCostReduction {
                    matching,
                    reduction: Energy(amount),
                }
            }
        })
        .boxed()
}

fn allied_spark_bonus<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    word("allied")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["have", "+"]))
        .then(spark())
        .then_ignore(word("spark"))
        .then_ignore(period())
        .map(|(matching, added_spark)| StandardStaticAbility::SparkBonusOtherCharacters {
            matching,
            added_spark: Spark(added_spark),
        })
}

fn enemy_cards_cost_increase<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["the", "opponent's"])
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(word("cost"))
        .then(energy())
        .then_ignore(word("more"))
        .then_ignore(period())
        .map(|(matching, increase)| StandardStaticAbility::EnemyCardsCostIncrease {
            matching,
            increase: Energy(increase),
        })
}

fn characters_in_hand_have_fast<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["characters", "in", "your", "hand", "have"])
        .ignore_then(directive("fast"))
        .ignore_then(period())
        .to(StandardStaticAbility::CharactersInHandHaveFast)
}

fn disable_enemy_materialized_abilities<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["disable", "the"])
        .ignore_then(directive("materialized"))
        .ignore_then(words(&["abilities", "of", "enemies"]))
        .ignore_then(period())
        .to(StandardStaticAbility::DisableEnemyMaterializedAbilities)
}

fn has_all_character_types<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["has", "all", "character", "types"])
        .ignore_then(period())
        .to(StandardStaticAbility::HasAllCharacterTypes)
}

fn abandon_ally_play_character_for_alternate_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    cost_parser::abandon_cost_single()
        .then_ignore(colon())
        .then_ignore(words(&["play", "this"]))
        .then_ignore(choice((word("character"), word("event"))))
        .then_ignore(word("for"))
        .then(energy())
        .then_ignore(comma())
        .then_ignore(words(&["then", "abandon", "it"]))
        .then_ignore(period())
        .map(|(additional_cost, e)| {
            StandardStaticAbility::PlayForAlternateCost(AlternateCost {
                energy_cost: Energy(e),
                additional_cost: Some(additional_cost),
                if_you_do: Some(Effect::Effect(StandardEffect::PayCost {
                    cost: Cost::AbandonCharactersCount {
                        target: Predicate::This,
                        count: CollectionExpression::Exactly(1),
                    },
                })),
            })
        })
}

fn play_for_alternate_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    choice((cost_parser::lose_maximum_energy_cost(), cost_parser::banish_from_hand_cost()))
        .then_ignore(colon())
        .then_ignore(words(&["play", "this"]))
        .then_ignore(choice((word("character"), word("event"))))
        .then_ignore(word("for"))
        .then(energy())
        .then_ignore(period())
        .map(|(additional_cost, e)| {
            StandardStaticAbility::PlayForAlternateCost(AlternateCost {
                energy_cost: Energy(e),
                additional_cost: Some(additional_cost),
                if_you_do: None,
            })
        })
}

fn simple_alternate_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    word("this")
        .ignore_then(choice((word("character"), word("event"))))
        .then_ignore(word("costs"))
        .ignore_then(energy())
        .map(|e| {
            StandardStaticAbility::PlayForAlternateCost(AlternateCost {
                energy_cost: Energy(e),
                additional_cost: None,
                if_you_do: None,
            })
        })
}

fn once_per_turn_play_from_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["once", "per", "turn"])
        .ignore_then(comma())
        .ignore_then(words(&["you", "may", "play"]))
        .ignore_then(article())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["from", "your", "void"]))
        .then_ignore(period())
        .map(|matching| StandardStaticAbility::OncePerTurnPlayFromVoid { matching })
}

fn additional_cost_to_play<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["to", "play", "this", "card"])
        .ignore_then(comma())
        .ignore_then(cost_parser::cost_parser())
        .then_ignore(period())
        .map(StandardStaticAbility::AdditionalCostToPlay)
}

fn reveal_top_card_of_deck<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["reveal", "the", "top", "card", "of", "your", "deck"])
        .ignore_then(period())
        .to(StandardStaticAbility::RevealTopCardOfYourDeck)
}

fn play_from_top_of_deck<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["you", "may", "play"])
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["from", "the", "top", "of", "your", "deck"]))
        .then_ignore(period())
        .map(|matching| StandardStaticAbility::YouMayPlayFromTopOfDeck { matching })
}

fn judgment_triggers_when_materialized<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["the", "'"])
        .ignore_then(directive("judgment"))
        .ignore_then(words(&["'", "ability", "of"]))
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["triggers", "when", "you"]))
        .then_ignore(directive("materialize"))
        .then_ignore(word("them"))
        .then_ignore(period())
        .map(|predicate| StandardStaticAbility::JudgmentTriggersWhenMaterialized { predicate })
}

fn spark_equal_to_predicate_count<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["this", "character's", "spark", "is", "equal", "to", "the", "number", "of"])
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(period())
        .map(|predicate| StandardStaticAbility::SparkEqualToPredicateCount { predicate })
}

fn play_only_from_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["you", "may", "only", "play", "this", "character", "from", "your", "void"])
        .ignore_then(period())
        .to(StandardStaticAbility::PlayOnlyFromVoid)
}

fn cards_in_your_void_have_reclaim<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["they", "have"])
        .ignore_then(directive("reclaim"))
        .ignore_then(words(&["equal", "to", "their", "cost"]))
        .ignore_then(period())
        .map(|_| StandardStaticAbility::CardsInYourVoidHaveReclaim {
            matching: CardPredicate::Card,
        })
}

fn play_from_hand_or_void_for_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardStaticAbility, ParserExtra<'a>> + Clone {
    words(&["you", "may", "play", "this", "card", "from", "your", "hand", "or", "void", "for"])
        .ignore_then(energy())
        .then_ignore(period())
        .map(|e| {
            StandardStaticAbility::PlayFromHandOrVoidForCost(PlayFromHandOrVoidForCost {
                energy_cost: Energy(e),
                additional_cost: None,
                if_you_do: None,
            })
        })
}
