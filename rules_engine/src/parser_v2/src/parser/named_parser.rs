use ability_data::named_ability::NamedAbility;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::parser_helpers::{ParserExtra, ParserInput};
use crate::variables::parser_substitutions::ResolvedToken;

pub fn named_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, NamedAbility, ParserExtra<'a>> + Clone {
    reclaim()
}

fn reclaim<'a>() -> impl Parser<'a, ParserInput<'a>, NamedAbility, ParserExtra<'a>> + Clone {
    reclaim_cost().map(|cost| NamedAbility::Reclaim(Some(Energy(cost))))
}

fn reclaim_cost<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _)
            if directive == "reclaimforcost" => value
    }
}
