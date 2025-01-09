use chumsky::prelude::*;

pub type ErrorType<'a> = extra::Err<Rich<'a, char>>;

pub fn phrase<'a>(text: &'static str) -> impl Parser<'a, &'a str, &'a str, ErrorType<'a>> {
    just(text).padded().boxed()
}

/// Standard identity function with a different name for readability
pub fn count(n: u64) -> u64 {
    n
}

/// Parses a numeric phrase, where the provided 'before' and 'after' text
/// surrounds an integer.
///
/// The integer can be mapped to a concrete type via the provided mapping
/// function, or you can pass [count] to use u64.
pub fn numeric<'a, T>(
    before: &'static str,
    function: impl Fn(u64) -> T + 'a,
    after: &'static str,
) -> impl Parser<'a, &'a str, T, ErrorType<'a>> {
    phrase(before)
        .ignore_then(text::int(10))
        .then_ignore(phrase(after))
        .map(move |s: &str| function(s.parse().unwrap()))
        .boxed()
}

/// Parses "this event" or "this character"
pub fn this<'a>() -> impl Parser<'a, &'a str, &'a str, ErrorType<'a>> {
    choice((phrase("this event"), phrase("this character"))).boxed()
}

/// Parses "a" or "an"
pub fn a_or_an<'a>() -> impl Parser<'a, &'a str, &'a str, ErrorType<'a>> {
    choice((phrase("an"), phrase("a"))).boxed()
}

/// Parses a number that can be either written as text (e.g. "two") or as a
/// numeral
pub fn text_number<'a>() -> impl Parser<'a, &'a str, u64, ErrorType<'a>> {
    choice((
        phrase("one").to(1),
        phrase("two").to(2),
        phrase("three").to(3),
        phrase("four").to(4),
        phrase("five").to(5),
        phrase("six").to(6),
        phrase("seven").to(7),
        phrase("eight").to(8),
        phrase("nine").to(9),
        text::int(10).from_str().unwrapped(),
    ))
    .boxed()
}

/// Parses an ordinal number that can be either written as text (e.g. "first")
/// or as a numeral with "st", "nd", "rd", or "th" suffix
pub fn ordinal_number<'a>() -> impl Parser<'a, &'a str, u64, ErrorType<'a>> {
    choice((
        phrase("first").to(1),
        phrase("second").to(2),
        phrase("third").to(3),
        phrase("fourth").to(4),
        phrase("fifth").to(5),
        phrase("sixth").to(6),
        phrase("seventh").to(7),
        phrase("eighth").to(8),
        phrase("ninth").to(9),
        numeric("", count, "th"),
    ))
    .boxed()
}
