use crate::lexer::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Directive(String),
    Period,
    Comma,
    Colon,
    Newline,
}

pub type Spanned<T> = (T, Span);
