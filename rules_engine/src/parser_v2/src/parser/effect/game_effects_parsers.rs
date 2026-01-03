use ability_data::collection_expression::CollectionExpression;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    article, directive, foresee_count, word, words, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, cost_parser, predicate_parser};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        choice((foresee(), discover(), counterspell_effects())).boxed(),
        choice((
            choice((dissolve_all_characters(), dissolve_character())).boxed(),
            choice((banish_collection(), banish_character(), banish_enemy_void())).boxed(),
            choice((materialize_copy(), materialize_character())).boxed(),
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

fn counterspell_effects<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((counterspell_unless_pays_cost(), counterspell())).boxed()
}
