use chumsky::extra::Err;
use chumsky::prelude::*;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;

use crate::lexer::lexer_token::Token;
use crate::variables::parser_substitutions::ResolvedToken;

pub type ParserInput<'a> = &'a [(ResolvedToken, SimpleSpan)];

pub type ParserExtra<'a> = Err<Rich<'a, (ResolvedToken, SimpleSpan), SimpleSpan>>;

pub fn word<'a>(
    text: &'static str,
) -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Token(Token::Word(w)), _) if w == text => ()
    }
}

pub fn directive<'a>(
    name: &'static str,
) -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Token(Token::Directive(d)), _) if d == name => ()
    }
}

pub fn period<'a>() -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Token(Token::Period), _) => ()
    }
}

pub fn comma<'a>() -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Token(Token::Comma), _) => ()
    }
}

pub fn colon<'a>() -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Token(Token::Colon), _) => ()
    }
}

pub fn effect_separator<'a>() -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    choice((period(), comma().then_ignore(word("then")), word("and")))
}

#[expect(clippy::unnested_or_patterns)]
pub fn energy<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Energy(v), _)
        | (ResolvedToken::Mode1Energy(v), _)
        | (ResolvedToken::Mode2Energy(v), _) => v,
    }
}

pub fn cards<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::CardCount(v), _) => v,
    }
}

/// Parses the {top_n_cards} directive value.
pub fn top_n_cards<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::TopNCards(v), _) => v,
    }
}

#[expect(clippy::unnested_or_patterns)]
pub fn discards<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::CardCount(v), _) | (ResolvedToken::DiscardCount(v), _) => v,
    }
}

pub fn points<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::PointCount(v), _) => v,
    }
}

pub fn spark<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::SparkAmount(v), _) => v,
    }
}

pub fn subtype<'a>() -> impl Parser<'a, ParserInput<'a>, CardSubtype, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Subtype(subtype), _) => subtype
    }
}

pub fn figment<'a>() -> impl Parser<'a, ParserInput<'a>, FigmentType, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Figment(figment_type), _) => figment_type
    }
}

pub fn figment_count<'a>(
) -> impl Parser<'a, ParserInput<'a>, (FigmentType, u32), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::FigmentCount { figment_type, count, .. }, _) => (figment_type, count)
    }
}

pub fn words<'a>(
    sequence: &'static [&'static str],
) -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    sequence.iter().fold(empty().boxed(), |acc, &w| acc.then_ignore(word(w)).boxed())
}

pub fn foresee_count<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::ForeseeCount(v), _) => v,
    }
}

pub fn kindle_amount<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::KindleAmount(v), _) => v,
    }
}

pub fn maximum_energy<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::MaximumEnergy(v), _) => v,
    }
}

pub fn article<'a>() -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    choice((word("a"), word("an")))
}

pub fn count<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Count(v), _) => v,
    }
}

pub fn count_allies<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::CountAllies(v), _) => v,
    }
}

pub fn up_to_n_allies<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::UpToNAllies(v), _) => v,
    }
}

pub fn it_or_them_count<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::ItOrThemCount(v), _) => v,
    }
}

pub fn count_allied_subtype<'a>(
) -> impl Parser<'a, ParserInput<'a>, (u32, CardSubtype), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::SubtypeCount { count, subtype, .. }, _) => (count, subtype)
    }
}

pub fn up_to_n_events<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::UpToNEvents(v), _) => v,
    }
}

pub fn number<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Number(v), _) => v,
    }
}

pub fn reclaim_cost<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::ReclaimCost(v), _) => v,
    }
}

pub fn this_turn_times<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::ThisTurnTimes(v), _) => v,
    }
}

pub fn mode1_cost<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Mode1Energy(v), _) => v,
    }
}

pub fn mode2_cost<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Mode2Energy(v), _) => v,
    }
}

pub fn newline<'a>() -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Token(Token::Newline), _) => ()
    }
}
