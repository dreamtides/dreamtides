use ability_data::collection_expression::CollectionExpression;
use chumsky::primitive::choice;
use chumsky::Parser;

use crate::parser_utils::{phrase, text_number, ErrorType};

pub fn parser<'a>() -> impl Parser<'a, &'a str, CollectionExpression, ErrorType<'a>> {
    choice((
        phrase("any number of").to(CollectionExpression::AnyNumberOf),
        phrase("all but one").to(CollectionExpression::AllButOne),
        phrase("up to").ignore_then(text_number()).map(CollectionExpression::UpTo),
        text_number().then_ignore(phrase("or more")).map(CollectionExpression::OrMore),
        text_number().map(CollectionExpression::Exactly),
        phrase("all").to(CollectionExpression::All),
        phrase("each other").to(CollectionExpression::EachOther),
        phrase("each").to(CollectionExpression::All),
    ))
    .boxed()
}
