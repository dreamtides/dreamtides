use chumsky::span::SimpleSpan;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Token {
    Word(String),
    Directive(String),
    Period,
    Comma,
    Colon,
    Newline,
}

pub type Spanned<T> = (T, SimpleSpan);
