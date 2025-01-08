use ability_data::counting_expression::CountingExpression;
use chumsky::primitive::choice;
use chumsky::Parser;

use crate::parser_utils::{phrase, text_number, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, CountingExpression, ErrorType<'a>> {
    choice((
        phrase("any number of").to(CountingExpression::AnyNumberOf),
        phrase("all but one").to(CountingExpression::AllButOne),
        phrase("up to").ignore_then(text_number()).map(CountingExpression::UpTo),
        text_number().then_ignore(phrase("or more")).map(CountingExpression::OrMore),
        text_number().map(CountingExpression::Exactly),
        phrase("all").to(CountingExpression::All),
    ))
    .boxed()
}
