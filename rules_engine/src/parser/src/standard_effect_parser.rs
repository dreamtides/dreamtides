use ability_data::collection_expression::CollectionExpression;
use ability_data::effect::Effect;
use ability_data::predicate::Predicate;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::triggered_ability::{TriggeredAbility, TriggeredAbilityOptions};
use chumsky::Parser;
use chumsky::prelude::*;
use core_data::numerics::{Energy, Points, Spark};

use crate::parser_utils::{
    ErrorType, a_or_an, a_or_count, card_or_cards, count, number_of_times, numeric, phrase,
    text_number,
};
use crate::{
    card_predicate_parser, collection_expression_parser, cost_parser, determiner_parser,
    quantity_expression_parser, trigger_event_parser,
};

/// Parses all standard game effects
pub fn parser<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    choice((non_recursive_effects(), create_trigger_until_end_of_turn())).boxed()
}

/// Parses all standard game effects that do not recursively invoke effect
/// parsing
fn non_recursive_effects<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    choice((
        card_effects(),
        spark_effects(),
        gain_effects(),
        enemy_effects(),
        game_state_effects(),
        pay_cost(),
    ))
    .boxed()
}

fn card_effects<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    choice((
        materialize_silent_copy(),
        draw_matching_card(),
        draw_cards_for_each(),
        draw_cards(),
        banish_card_from_enemy_void(),
        discard_card_from_enemy_hand_then_they_draw(),
        discard_card_from_enemy_hand(),
        return_all_but_one_character_draw_card_for_each(),
        put_on_top_of_deck(),
        put_cards_from_void_on_top_of_deck(),
        spend_all_energy_draw_and_discard(),
        spend_all_energy_dissolve_enemy(),
        materialize_character_from_void(),
        materialize_character_at_end_of_turn(),
        materialize_character(),
        dissolve_characters_count(),
        dissolve_characters_quantity(),
        return_to_hand(),
        copy(),
        copy_next_played(),
        shuffle_hand_and_deck_and_draw(),
        put_cards_from_deck_into_void(),
        each_player_discard_cards(),
        each_player_abandons_characters(),
    ))
    .boxed()
}

fn spark_effects<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    choice((
        gains_spark_for_quantity(),
        gain_spark_until_next_main_for_each(),
        gain_spark(),
        abandon_and_gain_energy_for_spark(),
        each_matching_gains_spark_for_each(),
        each_matching_gains_spark_until_next_main(),
        kindle(),
        spark_becomes(),
    ))
    .boxed()
}

fn gain_effects<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    choice((
        dissolve_character(),
        gains_aegis_this_turn(),
        gain_energy_for_each(),
        gain_energy(),
        double_your_energy(),
        gain_points_for_each(),
        gain_points(),
        gain_control(),
        foresee(),
        gain_twice_that_much_energy_instead(),
    ))
    .boxed()
}

fn enemy_effects<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    choice((
        lose_points(),
        enemy_gains_points_equal_to_its_spark(),
        enemy_gains_points(),
        enemy_loses_points(),
    ))
    .boxed()
}

fn game_state_effects<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    choice((
        disable_activated_abilities(),
        discover_and_then_materialize(),
        discover(),
        materialize_random_characters(),
        return_from_void_to_hand(),
        return_from_void_to_play(),
        gains_reclaim_until_end_of_turn(),
        cards_in_void_gain_reclaim_this_turn(),
        prevent_unless_pays_cost(),
        prevent(),
        abandon_at_end_of_turn(),
        banish_character_until_leaves_play(),
        banish_until_next_main(),
        banish_collection(),
        banish_character(),
        banish_enemy_void(),
        take_extra_turn(),
        trigger_judgment_ability(),
        win_game(),
    ))
    .boxed()
}

fn draw_cards<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("draw")
        .ignore_then(choice((phrase("a card").to(1), numeric("", count, "cards"))))
        .map(|count| StandardEffect::DrawCards { count })
}

fn gain_spark<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    determiner_parser::target_parser()
        .then(numeric("gains +", Spark, "spark"))
        .map(|(predicate, spark)| StandardEffect::GainsSpark { target: predicate, gains: spark })
}

fn gain_energy<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    numeric("gain $", Energy, "").map(|energy| StandardEffect::GainEnergy { gains: energy })
}

fn gain_spark_until_next_main_for_each<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    determiner_parser::target_parser()
        .then(numeric("gains +", Spark, "spark until your next main phase for each"))
        .then(card_predicate_parser::parser())
        .then_ignore(phrase("you control"))
        .map(|((target, spark), counted)| StandardEffect::GainsSparkUntilYourNextMainForEach {
            target,
            gains: spark,
            for_each: Predicate::Your(counted),
        })
}

fn dissolve_character<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("{dissolve}")
        .ignore_then(determiner_parser::target_parser())
        .map(|predicate| StandardEffect::DissolveCharacter { target: predicate })
}

fn gains_aegis_this_turn<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    determiner_parser::target_parser()
        .then_ignore(phrase("gains {kw: aegis} this turn"))
        .map(|target| StandardEffect::GainsAegisThisTurn { target })
}

fn draw_matching_card<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("draw a")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("from your deck"))
        .map(|card_predicate| StandardEffect::DrawMatchingCard { predicate: card_predicate })
}

fn banish_card_from_enemy_void<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("banish")
        .ignore_then(choice((phrase("a card").to(1), numeric("", count, "cards"))))
        .then_ignore(phrase("from the enemy's void"))
        .map(|count| StandardEffect::BanishCardsFromEnemyVoid { count })
}

fn disable_activated_abilities<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("disable the activated abilities of")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("while this character is in play"))
        .map(|target| StandardEffect::DisableActivatedAbilitiesWhileInPlay { target })
}

fn abandon_and_gain_energy_for_spark<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>>
{
    phrase("abandon")
        .ignore_then(determiner_parser::your_action())
        .then(numeric("and gain $", Energy, ""))
        .then_ignore(phrase("for each point of spark that character had"))
        .map(|(predicate, energy)| StandardEffect::AbandonAndGainEnergyForSpark {
            target: predicate,
            energy_per_spark: energy,
        })
}

fn gain_energy_for_each<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    numeric("gain $", Energy, "").then(determiner_parser::for_each_parser()).map(
        |(gained, counted)| StandardEffect::GainEnergyForEach { gains: gained, for_each: counted },
    )
}

fn create_trigger_until_end_of_turn<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>>
{
    phrase("until end of turn, whenever")
        .ignore_then(trigger_event_parser::event_parser())
        .then_ignore(phrase(","))
        .then(non_recursive_effects())
        .map(move |(trigger, effect)| StandardEffect::CreateTriggerUntilEndOfTurn {
            trigger: Box::new(TriggeredAbility {
                trigger,
                effect: Effect::Effect(effect),
                options: Some(TriggeredAbilityOptions {
                    once_per_turn: false,
                    until_end_of_turn: true,
                }),
            }),
        })
}

fn discover<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("{kw: discover}")
        .ignore_then(a_or_an())
        .ignore_then(card_predicate_parser::parser())
        .map(|predicate| StandardEffect::Discover { predicate })
}

fn discover_and_then_materialize<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("{kw: discover}")
        .ignore_then(choice((phrase("a"), phrase("an"))))
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("and materialize it"))
        .map(|predicate| StandardEffect::DiscoverAndThenMaterialize { predicate })
}

fn materialize_random_characters<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("materialize")
        .ignore_then(choice((
            phrase("a random").to(1),
            text_number().then_ignore(phrase("random")),
        )))
        .then(card_predicate_parser::parser())
        .then_ignore(phrase("from your deck"))
        .map(|(count, predicate)| StandardEffect::MaterializeRandomFromDeck { count, predicate })
}

fn return_from_void_to_hand<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("return")
        .ignore_then(determiner_parser::your_action())
        .then_ignore(phrase("from your void to your hand"))
        .map(|target| StandardEffect::ReturnFromYourVoidToHand { target })
}

fn return_from_void_to_play<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("return")
        .ignore_then(determiner_parser::your_action())
        .then_ignore(phrase("from your void to play"))
        .map(|target| StandardEffect::ReturnFromYourVoidToPlay { target })
}

fn gains_reclaim_until_end_of_turn<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>>
{
    determiner_parser::target_parser()
        .then_ignore(phrase("gains {kw: reclaim}"))
        .then(numeric("$", Energy, "").or_not())
        .then_ignore(phrase("until end of turn"))
        .map(|(target, cost)| StandardEffect::GainsReclaimUntilEndOfTurn { target, cost })
}

fn kindle<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    numeric("{kw: kindle}", Spark, "").map(|amount| StandardEffect::Kindle { amount })
}

fn prevent_unless_pays_cost<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("{prevent}")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("unless the enemy"))
        .then(cost_parser::third_person_singular_present_tense_cost())
        .map(|(target, cost)| StandardEffect::CounterspellUnlessPaysCost { target, cost })
}

fn prevent<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("{prevent}")
        .ignore_then(determiner_parser::target_parser())
        .map(|target| StandardEffect::Counterspell { target })
}

fn discard_card_from_enemy_hand<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("look at the enemy's hand. choose")
        .ignore_then(a_or_an())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("from it. the enemy discards that card"))
        .map(|predicate| StandardEffect::DiscardCardFromEnemyHand { predicate })
}

fn discard_card_from_enemy_hand_then_they_draw<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("look at the enemy's hand. you may choose")
        .ignore_then(a_or_an())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("from it. the enemy discards that card and then draws a card"))
        .map(|predicate| StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate })
}

fn abandon_at_end_of_turn<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("abandon")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("at end of turn"))
        .map(|target| StandardEffect::AbandonAtEndOfTurn { target })
}

fn spend_all_energy_draw_and_discard<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>>
{
    phrase("spend all your remaining energy. draw x cards then discard x cards, where x is the energy spent this way")
        .to(StandardEffect::SpendAllEnergyDrawAndDiscard)
}

fn spend_all_energy_dissolve_enemy<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>>
{
    phrase("spend all your remaining energy. {dissolve} an enemy character with spark less than or equal to the energy spent this way")
        .to(StandardEffect::SpendAllEnergyDissolveEnemy)
}

fn put_on_top_of_deck<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("put")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("on top of the enemy's deck"))
        .map(|target| StandardEffect::PutOnTopOfEnemyDeck { target })
}

fn each_matching_gains_spark_until_next_main<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("each")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("you control gains +"))
        .then(numeric("", Spark, "spark until your next main phase"))
        .map(|(each, gains)| StandardEffect::EachMatchingGainsSparkUntilNextMain { each, gains })
}

fn each_matching_gains_spark_for_each<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("each")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("you control gains +x spark, where x is the number of"))
        .then(card_predicate_parser::parser())
        .then_ignore(phrase("you control"))
        .map(|(matching, for_each)| StandardEffect::EachMatchingGainsSparkForEach {
            each: matching,
            gains: Spark(1),
            for_each,
        })
}

fn return_all_but_one_character_draw_card_for_each<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("return")
        .ignore_then(collection_expression_parser::parser())
        .then_ignore(phrase(
            "character you control to hand. draw a card for each character returned",
        ))
        .map(|count| StandardEffect::ReturnCharactersToHandDrawCardForEach { count })
}

fn banish_character<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("banish")
        .ignore_then(determiner_parser::target_parser())
        .map(|predicate| StandardEffect::BanishCharacter { target: predicate })
}

fn banish_collection<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("banish")
        .ignore_then(collection_expression_parser::parser())
        .then(determiner_parser::counted_parser())
        .map(|(collection, target)| StandardEffect::BanishCollection { target, count: collection })
}

fn materialize_character_from_void<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>>
{
    phrase("materialize")
        .ignore_then(a_or_an())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("from your void"))
        .map(|target| StandardEffect::MaterializeCharacterFromVoid { target })
}

fn materialize_character<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("materialize")
        .ignore_then(determiner_parser::target_parser())
        .map(|target| StandardEffect::MaterializeCharacter { target })
}

fn materialize_character_at_end_of_turn<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("materialize")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("at end of turn"))
        .map(|target| StandardEffect::MaterializeCharacterAtEndOfTurn { target })
}

fn gain_points<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    numeric("gain", Points, "$point")
        .then_ignore(just("s").or_not())
        .map(|points| StandardEffect::GainPoints { gains: points })
}

fn foresee<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    numeric("{kw: foresee}", count, "").map(|count| StandardEffect::Foresee { count })
}

fn lose_points<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    numeric("you lose", Points, "$point")
        .then_ignore(just("s").or_not())
        .map(|points| StandardEffect::LosePoints { loses: points })
}

fn dissolve_characters_count<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("{dissolve}")
        .ignore_then(collection_expression_parser::parser())
        .then(determiner_parser::counted_parser())
        .map(|(count, target)| StandardEffect::DissolveCharactersCount { target, count })
}

fn enemy_gains_points<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("the enemy gains")
        .ignore_then(numeric("", count, "$point"))
        .then_ignore(just("s").or_not())
        .map(|count| StandardEffect::EnemyGainsPoints { count })
}

fn enemy_gains_points_equal_to_its_spark<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("the enemy gains $points equal to its spark")
        .to(StandardEffect::EnemyGainsPointsEqualToItsSpark)
}

fn enemy_loses_points<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("the enemy loses")
        .ignore_then(numeric("", count, "$point"))
        .then_ignore(just("s").or_not())
        .map(|count| StandardEffect::EnemyLosesPoints { count })
}

fn pay_cost<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    cost_parser::standard_cost().map(|cost| StandardEffect::PayCost { cost })
}

fn gain_control<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("gain control of")
        .ignore_then(determiner_parser::target_parser())
        .map(|target| StandardEffect::GainControl { target })
}

fn return_to_hand<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("return")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("to hand"))
        .map(|target| StandardEffect::ReturnToHand { target })
}

fn gain_points_for_each<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("gain")
        .ignore_then(numeric("", Points, "$point"))
        .then_ignore(just("s").or_not())
        .then_ignore(phrase("for each"))
        .then(quantity_expression_parser::parser())
        .map(|(gain, for_count)| StandardEffect::GainPointsForEach { gain, for_count })
}

fn draw_cards_for_each<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("draw")
        .ignore_then(a_or_count())
        .then_ignore(card_or_cards())
        .then_ignore(phrase("for each"))
        .then(quantity_expression_parser::parser())
        .map(|(count, for_count)| StandardEffect::DrawCardsForEach { count, for_each: for_count })
}

fn copy<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    choice((phrase("copy"), phrase("materialize a copy of")))
        .ignore_then(determiner_parser::target_parser())
        .map(|target| StandardEffect::Copy { target })
}

fn copy_next_played<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("copy the next")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(phrase("you play this turn"))
        .then(number_of_times())
        .map(|(matching, times)| StandardEffect::CopyNextPlayed {
            matching: Predicate::Your(matching),
            times,
        })
}

fn cards_in_void_gain_reclaim_this_turn<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("until end of turn,")
        .ignore_then(collection_expression_parser::parser())
        .then(card_predicate_parser::parser())
        .then_ignore(phrase("in your void have {kw: reclaim}"))
        .map(|(count, predicate)| StandardEffect::CardsInVoidGainReclaimThisTurn {
            count,
            predicate,
        })
}

fn gains_spark_for_quantity<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    determiner_parser::target_parser()
        .then(numeric("gains +", Spark, "spark for each"))
        .then(quantity_expression_parser::parser())
        .map(|((target, gains), for_quantity)| StandardEffect::GainsSparkForQuantity {
            target,
            gains,
            for_quantity,
        })
}

fn take_extra_turn<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("take an extra turn after this one").to(StandardEffect::TakeExtraTurn)
}

fn double_your_energy<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("double the amount of energy in your energy pool").to(StandardEffect::DoubleYourEnergy)
}

fn gain_twice_that_much_energy_instead<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("gain twice that much energy instead").to(StandardEffect::GainTwiceThatMuchEnergyInstead)
}

fn shuffle_hand_and_deck_and_draw<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("each player may shuffle their hand and void into their deck and then draw")
        .ignore_then(numeric("", count, "cards"))
        .map(|count| StandardEffect::ShuffleHandAndDeckAndDraw { count })
}

fn dissolve_characters_quantity<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("{dissolve}")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("with cost less than or equal to the number of"))
        .then(quantity_expression_parser::parser())
        .map(|(target, quantity)| StandardEffect::DissolveCharactersQuantity { target, quantity })
}

fn put_cards_from_deck_into_void<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    numeric("put the top", count, "cards of your deck into your void")
        .map(|count| StandardEffect::PutCardsFromYourDeckIntoVoid { count })
}

fn trigger_judgment_ability<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("trigger the '$judgment' ability of")
        .ignore_then(collection_expression_parser::parser().or_not())
        .then(determiner_parser::counted_parser())
        .map(|(collection, matching)| StandardEffect::TriggerJudgmentAbility {
            matching,
            collection: collection.unwrap_or(CollectionExpression::All),
        })
}

fn win_game<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("you win the game").to(StandardEffect::YouWinTheGame)
}

fn banish_enemy_void<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("banish the enemy's void").to(StandardEffect::BanishEnemyVoid)
}

fn spark_becomes<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("the spark of")
        .ignore_then(collection_expression_parser::parser().or_not())
        .then(card_predicate_parser::parser())
        .then(numeric("you control becomes", Spark, ""))
        .map(|((collection, matching), spark)| StandardEffect::SparkBecomes {
            collection: collection.unwrap_or(CollectionExpression::All),
            matching,
            spark,
        })
}

fn banish_character_until_leaves_play<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("banish")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("until"))
        .then(determiner_parser::target_parser())
        .then_ignore(phrase("leaves play"))
        .map(|(target, until_leaves)| StandardEffect::BanishCharacterUntilLeavesPlay {
            target,
            until_leaves,
        })
}

fn banish_until_next_main<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("banish")
        .ignore_then(determiner_parser::target_parser())
        .then_ignore(phrase("until the start of your next main phase"))
        .map(|target| StandardEffect::BanishUntilNextMain { target })
}

fn each_player_discard_cards<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("each player discards")
        .ignore_then(a_or_count())
        .then_ignore(card_or_cards())
        .map(|count| StandardEffect::EachPlayerDiscardCards { count })
}

fn put_cards_from_void_on_top_of_deck<'a>()
-> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("put")
        .ignore_then(a_or_count())
        .then(card_predicate_parser::parser())
        .then_ignore(phrase("from your void on top of your deck"))
        .map(|(count, matching)| StandardEffect::PutCardsFromVoidOnTopOfDeck { count, matching })
}

fn materialize_silent_copy<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>> {
    phrase("materialize").ignore_then(
        choice((
            phrase("a {kw: silent} copy").to(1),
            text_number().then_ignore(phrase("{kw: silent} copies")).map(|n| n),
        ))
        .then_ignore(phrase("of"))
        .then(determiner_parser::target_parser())
        .then(phrase("for each").ignore_then(quantity_expression_parser::parser()).or_not())
        .map(|((count, target), quantity)| {
            let target_clone = target.clone();
            StandardEffect::MaterializeSilentCopy {
                target,
                count,
                quantity: quantity.unwrap_or(QuantityExpression::Matching(target_clone)),
            }
        }),
    )
}

fn each_player_abandons_characters<'a>() -> impl Parser<'a, &'a str, StandardEffect, ErrorType<'a>>
{
    phrase("each player abandons")
        .ignore_then(a_or_count())
        .then(card_predicate_parser::parser())
        .map(|(count, matching)| StandardEffect::EachPlayerAbandonsCharacters { matching, count })
}
