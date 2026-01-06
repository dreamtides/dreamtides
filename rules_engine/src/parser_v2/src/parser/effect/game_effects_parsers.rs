use ability_data::collection_expression::CollectionExpression;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    article, cards, comma, directive, figment, figment_count, foresee_count, it_or_them_count,
    number, this_turn_times, up_to_n_allies, word, words, ParserExtra, ParserInput,
};
use crate::parser::{
    card_predicate_parser, cost_parser, predicate_parser, quantity_expression_parser,
};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        choice((
            you_win_the_game(),
            copy_next_played(),
            copy_it(),
            foresee(),
            each_player_shuffles_hand_and_void_and_draws(),
            discover_and_materialize(),
            discover(),
            counterspell_effects(),
            trigger_judgment_ability(),
        ))
        .boxed(),
        choice((
            choice((dissolve_each_character(), dissolve_all_characters(), dissolve_character()))
                .boxed(),
            choice((
                banish_cards_from_opponent_void(),
                banish_up_to_n(),
                banish_collection(),
                banish_character_until_leaves_play(),
                banish_until_next_main(),
                banish_character(),
                banish_enemy_void(),
            ))
            .boxed(),
            choice((
                materialize_character_at_end_of_turn(),
                materialize_random_from_deck(),
                materialize_collection(),
                materialize_copy(),
                materialize_figments_quantity(),
                materialize_figments(),
                materialize_character(),
            ))
            .boxed(),
            trigger_additional_judgment_phase(),
            take_extra_turn(),
        ))
        .boxed(),
    ))
    .boxed()
}

pub fn foresee<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    foresee_count().map(|count| StandardEffect::Foresee { count })
}

pub fn each_player_shuffles_hand_and_void_and_draws<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["each", "player", "shuffles", "their", "hand", "and", "void", "into", "their", "deck"])
        .ignore_then(words(&["and", "then", "draws"]))
        .ignore_then(cards())
        .map(|count| StandardEffect::EachPlayerShufflesHandAndVoidIntoDeckAndDraws { count })
}

pub fn discover<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("discover")
        .ignore_then(article().or_not())
        .ignore_then(card_predicate_parser::parser())
        .map(|predicate| StandardEffect::Discover { predicate })
}

pub fn discover_and_materialize<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("discover")
        .ignore_then(article().or_not())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(word("and"))
        .then_ignore(directive("materialize"))
        .then_ignore(word("it"))
        .map(|predicate| StandardEffect::DiscoverAndThenMaterialize { predicate })
}

pub fn counterspell<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("prevent")
        .ignore_then(words(&["a", "played"]))
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::Counterspell { target })
}

pub fn counterspell_unless_pays_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("prevent")
        .ignore_then(words(&["a", "played"]))
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["unless", "the", "opponent", "pays"]))
        .then(cost_parser::cost_parser())
        .map(|(target, cost)| StandardEffect::CounterspellUnlessPaysCost { target, cost })
}

pub fn dissolve_each_character<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("dissolve")
        .ignore_then(word("each"))
        .ignore_then(card_predicate_parser::parser())
        .map(|predicate| StandardEffect::DissolveCharactersCount {
            target: Predicate::Any(predicate),
            count: CollectionExpression::All,
        })
}

pub fn dissolve_all_characters<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("dissolve").ignore_then(word("all")).ignore_then(word("characters")).map(|_| {
        StandardEffect::DissolveCharactersCount {
            target: Predicate::Any(CardPredicate::Character),
            count: CollectionExpression::All,
        }
    })
}

pub fn dissolve_character<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("dissolve")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::DissolveCharacter { target })
}

pub fn banish_character<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::BanishCharacter { target })
}

pub fn banish_character_until_leaves_play<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(word("until"))
        .then(predicate_parser::predicate_parser())
        .then_ignore(words(&["leaves", "play"]))
        .map(|(target, until_leaves)| StandardEffect::BanishCharacterUntilLeavesPlay {
            target,
            until_leaves,
        })
}

pub fn banish_until_next_main<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["until", "your", "next", "main", "phase"]))
        .map(|target| StandardEffect::BanishUntilNextMain { target })
}

pub fn banish_collection<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(words(&["any", "number", "of"]))
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::BanishCollection {
            target,
            count: CollectionExpression::AnyNumberOf,
        })
}

pub fn banish_up_to_n<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish").ignore_then(up_to_n_allies()).map(|count| {
        StandardEffect::BanishCollection {
            target: Predicate::Another(CardPredicate::Character),
            count: CollectionExpression::UpTo(count),
        }
    })
}

pub fn banish_cards_from_opponent_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(cards())
        .then_ignore(words(&["from", "the", "opponent's", "void"]))
        .map(|count| StandardEffect::BanishCardsFromEnemyVoid { count })
}

pub fn banish_enemy_void<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(words(&["the", "opponent's", "void"]))
        .map(|_| StandardEffect::BanishEnemyVoid)
}

pub fn materialize_character<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("materialize")
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::MaterializeCharacter { target })
}

pub fn materialize_character_at_end_of_turn<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("materialize")
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["at", "end", "of", "turn"]))
        .map(|target| StandardEffect::MaterializeCharacterAtEndOfTurn { target })
}

pub fn materialize_collection<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("materialize").ignore_then(it_or_them_count()).map(|_count| {
        StandardEffect::MaterializeCollection {
            target: Predicate::Them,
            count: CollectionExpression::All,
        }
    })
}

pub fn materialize_copy<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("materialize")
        .ignore_then(words(&["a", "copy", "of"]))
        .ignore_then(article().or_not())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| StandardEffect::MaterializeSilentCopy {
            target: target.clone(),
            count: 1,
            quantity: QuantityExpression::Matching(target),
        })
}

pub fn materialize_figments<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("materialize").ignore_then(choice((
        figment_count()
            .map(|(figment, count)| StandardEffect::MaterializeFigments { figment, count }),
        figment().map(|figment| StandardEffect::MaterializeFigments { figment, count: 1 }),
    )))
}

pub fn materialize_random_from_deck<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("materialize")
        .ignore_then(number())
        .then(card_predicate_parser::parser())
        .then_ignore(words(&["from", "your", "deck"]))
        .map(|(count, predicate)| StandardEffect::MaterializeRandomFromDeck { count, predicate })
}

pub fn materialize_figments_quantity<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("materialize")
        .ignore_then(figment())
        .then_ignore(words(&["for", "each"]))
        .then(quantity_expression_parser::parser())
        .map(|(figment, quantity)| StandardEffect::MaterializeFigmentsQuantity {
            figment,
            count: 1,
            quantity,
        })
}

pub fn copy_next_played<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("copy")
        .ignore_then(words(&["the", "next"]))
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["you", "play"]))
        .then(this_turn_times())
        .map(|(card_predicate, times)| StandardEffect::CopyNextPlayed {
            matching: Predicate::Your(card_predicate),
            times: Some(times),
        })
}

pub fn copy_it<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("copy").ignore_then(word("it")).to(StandardEffect::Copy { target: Predicate::It })
}

pub fn trigger_additional_judgment_phase<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["at", "the", "end", "of", "this", "turn"])
        .ignore_then(comma())
        .ignore_then(words(&["trigger", "an", "additional"]))
        .ignore_then(directive("judgmentphasename"))
        .ignore_then(word("phase"))
        .to(StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn)
}

pub fn take_extra_turn<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["take", "an", "extra", "turn", "after", "this", "one"])
        .to(StandardEffect::TakeExtraTurn)
}

pub fn trigger_judgment_ability<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["trigger", "the"])
        .ignore_then(directive("judgment"))
        .ignore_then(words(&["ability", "of", "each", "ally"]))
        .to(StandardEffect::TriggerJudgmentAbility {
            matching: Predicate::Another(CardPredicate::Character),
            collection: CollectionExpression::All,
        })
}

pub fn you_win_the_game<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["you", "win", "the", "game"]).to(StandardEffect::YouWinTheGame)
}

fn counterspell_effects<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((counterspell_unless_pays_cost(), counterspell())).boxed()
}
