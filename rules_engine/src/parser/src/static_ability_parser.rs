use ability_data::effect::Effect;
use ability_data::predicate::CardPredicate;
use ability_data::static_ability::{
    AlternateCost, PlayFromVoid, StandardStaticAbility, StaticAbility, StaticAbilityWithOptions,
};
use chumsky::Parser;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Spark};

use crate::parser_utils::{ErrorType, numeric, phrase, this};
use crate::{
    card_predicate_parser, condition_parser, cost_parser, determiner_parser,
    quantity_expression_parser, standard_effect_parser,
};

pub fn parser<'a>() -> impl Parser<'a, &'a str, StaticAbility, ErrorType<'a>> {
    choice((
        phrase("if")
            .ignore_then(condition_parser::parser())
            .then_ignore(phrase(","))
            .then(standard())
            .map(|(condition, ability)| {
                StaticAbility::WithOptions(StaticAbilityWithOptions {
                    ability,
                    condition: Some(condition),
                })
            }),
        standard().map(StaticAbility::StaticAbility),
    ))
    .boxed()
}

fn standard<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    choice((
        cards_in_your_void_have_reclaim(),
        cost_increase(),
        cost_reduction(),
        cost_reduction_for_each(),
        disable_enemy_materialized_abilities(),
        once_per_turn_play_from_void(),
        enemy_added_cost_to_play(),
        other_spark_bonus(),
        your_characters_spark_bonus(),
        has_all_character_types(),
        play_from_void(),
        simple_alternate_cost(),
        play_for_alternate_cost(),
        play_only_from_void(),
        spark_equal_to_predicate_count(),
        characters_in_hand_have_fast(),
        judgment_triggers_when_materialized(),
        look_at_top_card(),
        play_from_top_of_deck(),
    ))
    .boxed()
}

fn once_per_turn_play_from_void<'a>()
-> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("once per turn, you may play a")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("from your void"))
        .map(|matching| StandardStaticAbility::OncePerTurnPlayFromVoid { matching })
}

fn enemy_added_cost_to_play<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>>
{
    phrase("the enemy's")
        .ignore_then(card_predicate_parser::parser())
        .then(numeric("cost $", Energy, "more"))
        .map(|(predicate, cost)| StandardStaticAbility::EnemyCardsCostIncrease {
            matching: predicate,
            increase: cost,
        })
}

fn disable_enemy_materialized_abilities<'a>()
-> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    let enemy_characters = choice((phrase("the enemy's characters"), phrase("enemy characters")));
    phrase("disable the \"{materialized}\" abilities of")
        .ignore_then(enemy_characters)
        .to(StandardStaticAbility::DisableEnemyMaterializedAbilities)
}

fn other_spark_bonus<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("other")
        .ignore_then(card_predicate_parser::parser())
        .then(numeric("you control have +", Spark, "spark"))
        .map(|(predicate, spark)| StandardStaticAbility::SparkBonusOtherCharacters {
            matching: predicate,
            added_spark: spark,
        })
}

fn your_characters_spark_bonus<'a>()
-> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    card_predicate_parser::parser().then(numeric("you control have +", Spark, "spark")).map(
        |(predicate, spark)| StandardStaticAbility::SparkBonusYourCharacters {
            matching: predicate,
            added_spark: spark,
        },
    )
}

fn has_all_character_types<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("this character has all character types").to(StandardStaticAbility::HasAllCharacterTypes)
}

fn cost_increase<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    card_predicate_parser::parser().then(numeric("cost you $", Energy, "more")).map(
        |(predicate, cost)| StandardStaticAbility::YourCardsCostIncrease {
            matching: predicate,
            reduction: cost,
        },
    )
}

fn cost_reduction<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    card_predicate_parser::parser().then(numeric("cost you $", Energy, "less")).map(
        |(predicate, cost)| StandardStaticAbility::YourCardsCostReduction {
            matching: predicate,
            reduction: cost,
        },
    )
}

fn play_from_void<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("you may play")
        .ignore_then(this())
        .ignore_then(phrase("from your void"))
        .ignore_then(numeric("for $", Energy, "").or_not())
        .then(phrase("by").ignore_then(cost_parser::present_participle_additional_cost()).or_not())
        .then(phrase(". if you do,").ignore_then(standard_effect_parser::parser()).or_not())
        .map(|((energy_cost, additional_cost), if_you_do)| {
            StandardStaticAbility::PlayFromVoid(PlayFromVoid {
                energy_cost,
                additional_cost,
                if_you_do: if_you_do.map(Effect::Effect),
            })
        })
}

fn play_for_alternate_cost<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("you may play")
        .ignore_then(this())
        .ignore_then(numeric("for $", Energy, ""))
        .then(phrase("by").ignore_then(cost_parser::present_participle_additional_cost()).or_not())
        .then(phrase(". if you do,").ignore_then(standard_effect_parser::parser()).or_not())
        .map(|((energy_cost, additional_cost), if_you_do)| {
            StandardStaticAbility::PlayForAlternateCost(AlternateCost {
                energy_cost,
                additional_cost,
                if_you_do: if_you_do.map(Effect::Effect),
            })
        })
}

fn simple_alternate_cost<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    this().ignore_then(numeric("costs $", Energy, "")).map(|energy_cost| {
        StandardStaticAbility::PlayForAlternateCost(AlternateCost {
            energy_cost,
            additional_cost: None,
            if_you_do: None,
        })
    })
}

fn spark_equal_to_predicate_count<'a>()
-> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("this character's spark is equal to the number of")
        .ignore_then(determiner_parser::counted_parser())
        .map(|predicate| StandardStaticAbility::SparkEqualToPredicateCount { predicate })
}

fn characters_in_hand_have_fast<'a>()
-> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("characters in your hand have '$fast'")
        .to(StandardStaticAbility::CharactersInHandHaveFast)
}

fn judgment_triggers_when_materialized<'a>()
-> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("the '$judgment' ability of")
        .ignore_then(determiner_parser::counted_parser())
        .then_ignore(phrase("triggers when you materialize them"))
        .map(|predicate| StandardStaticAbility::JudgmentTriggersWhenMaterialized { predicate })
}

fn look_at_top_card<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("you may look at the top card of your deck")
        .to(StandardStaticAbility::YouMayLookAtTopCardOfYourDeck)
}

fn play_from_top_of_deck<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("you may play")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("from the top of your deck"))
        .map(|matching| StandardStaticAbility::YouMayPlayFromTopOfDeck { matching })
}

fn play_only_from_void<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("you may only play this character from your void")
        .to(StandardStaticAbility::PlayOnlyFromVoid)
}

fn cards_in_your_void_have_reclaim<'a>()
-> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    phrase("cards in your void have {kw: reclaim}").map(|_| {
        StandardStaticAbility::CardsInYourVoidHaveReclaim { matching: CardPredicate::Card }
    })
}

fn cost_reduction_for_each<'a>() -> impl Parser<'a, &'a str, StandardStaticAbility, ErrorType<'a>> {
    this()
        .ignore_then(numeric("costs $", Energy, "less to play for each"))
        .then(quantity_expression_parser::parser())
        .map(|(reduction, quantity)| StandardStaticAbility::CostReductionForEach {
            reduction,
            quantity,
        })
}
