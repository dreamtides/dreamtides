use ability_data::collection_expression::CollectionExpression;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    article, cards, directive, figment, figment_count, foresee_count, it_or_them_count,
    up_to_n_allies, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, cost_parser, predicate_parser};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        choice((foresee(), discover_and_materialize(), discover(), counterspell_effects())).boxed(),
        choice((
            choice((dissolve_all_characters(), dissolve_character())).boxed(),
            choice((
                banish_cards_from_opponent_void(),
                banish_up_to_n(),
                banish_collection(),
                banish_character(),
                banish_enemy_void(),
            ))
            .boxed(),
            choice((
                materialize_character_at_end_of_turn(),
                materialize_collection(),
                materialize_copy(),
                materialize_figments_quantity(),
                materialize_figments(),
                materialize_character(),
            ))
            .boxed(),
        ))
        .boxed(),
    ))
    .boxed()
}

pub fn foresee<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    foresee_count().map(|count| StandardEffect::Foresee { count })
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
    directive("materialize")
        .ignore_then(figment_count())
        .map(|(figment, count)| StandardEffect::MaterializeFigments { figment, count })
}

pub fn materialize_figments_quantity<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    directive("materialize")
        .ignore_then(figment())
        .then_ignore(words(&["for", "each"]))
        .then(card_predicate_parser::parser())
        .then_ignore(words(&["you", "have", "played", "this", "turn"]))
        .map(|(figment, predicate)| StandardEffect::MaterializeFigmentsQuantity {
            figment,
            count: 1,
            quantity: QuantityExpression::PlayedThisTurn(predicate),
        })
}

fn counterspell_effects<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((counterspell_unless_pays_cost(), counterspell())).boxed()
}
