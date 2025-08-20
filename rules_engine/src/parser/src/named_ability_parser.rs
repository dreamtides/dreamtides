use ability_data::named_ability::NamedAbility;
use chumsky::Parser;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser_utils::{ErrorType, numeric, phrase};

pub fn parser<'a>() -> impl Parser<'a, &'a str, NamedAbility, ErrorType<'a>> {
    reclaim().boxed()
}

fn reclaim<'a>() -> impl Parser<'a, &'a str, NamedAbility, ErrorType<'a>> {
    choice((
        phrase("{-reclaim}").to(NamedAbility::Reclaim(None)),
        numeric("{-reclaim-cost(e:", Energy, ")}").map(|e| NamedAbility::Reclaim(Some(e))),
    ))
}
