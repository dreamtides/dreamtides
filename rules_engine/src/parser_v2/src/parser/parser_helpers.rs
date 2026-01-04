use chumsky::extra::Err;
use chumsky::prelude::*;
use core_data::card_types::CardSubtype;

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

pub fn energy<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "e" => value
    }
}

pub fn cards<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "cards" => value
    }
}

/// Parses the {cards-numeral} directive value.
pub fn cards_numeral<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "cards-numeral" => value
    }
}

/// Parses the {top-n-cards} directive value.
pub fn top_n_cards<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "top-n-cards" => value
    }
}

pub fn discards<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "discards" => value
    }
}

pub fn points<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "points" => value
    }
}

pub fn spark<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "s" => value
    }
}

pub fn subtype<'a>() -> impl Parser<'a, ParserInput<'a>, CardSubtype, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Subtype { subtype, .. }, _) => subtype
    }
}

pub fn words<'a>(
    sequence: &'static [&'static str],
) -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    sequence.iter().fold(empty().boxed(), |acc, &w| acc.then_ignore(word(w)).boxed())
}

pub fn foresee_count<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "foresee" || directive == "Foresee" => value
    }
}

pub fn kindle_amount<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "kindle" || directive == "Kindle" => value
    }
}

pub fn article<'a>() -> impl Parser<'a, ParserInput<'a>, (), ParserExtra<'a>> + Clone {
    choice((word("a"), word("an")))
}

pub fn count_allies<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "count-allies" => value
    }
}

pub fn up_to_n_allies<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "up-to-n-allies" => value
    }
}

pub fn it_or_them_count<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _) if directive == "it-or-them" => value
    }
}

pub fn count_allied_subtype<'a>(
) -> impl Parser<'a, ParserInput<'a>, (u32, CardSubtype), ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::SubtypeCount { count, subtype, .. }, _) => (count, subtype)
    }
}
