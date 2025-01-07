use chumsky::prelude::*;

pub type ErrorType<'a> = extra::Err<Rich<'a, char>>;

pub fn phrase<'a>(text: &'static str) -> impl Parser<'a, &'a str, &'a str, ErrorType<'a>> {
    just(text).padded()
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
}
