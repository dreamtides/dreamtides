use ability_data::named_ability::NamedAbility;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::lexer::lexer_token::Token;
use crate::parser::cost_parser;
use crate::parser::parser_helpers::{word, ParserExtra, ParserInput};
use crate::variables::parser_substitutions::ResolvedToken;

pub fn named_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, NamedAbility, ParserExtra<'a>> + Clone {
    choice((reclaim_for_cost(), reclaim())).boxed()
}

fn reclaim<'a>() -> impl Parser<'a, ParserInput<'a>, NamedAbility, ParserExtra<'a>> + Clone {
    reclaim_cost().map(|cost| NamedAbility::Reclaim(Some(Energy(cost))))
}

fn reclaim_for_cost<'a>() -> impl Parser<'a, ParserInput<'a>, NamedAbility, ParserExtra<'a>> + Clone
{
    reclaim_directive()
        .ignore_then(word("--"))
        .ignore_then(cost_parser::cost_parser())
        .map(NamedAbility::ReclaimForCost)
}

fn reclaim_directive<'a>() -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Token(Token::Directive(d)), _) if d == "reclaim" => ()
    }
}

fn reclaim_cost<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _)
            if directive == "reclaimforcost" => value
    }
}
