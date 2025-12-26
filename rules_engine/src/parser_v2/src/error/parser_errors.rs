use chumsky::error::Rich;
use chumsky::span::SimpleSpan;

use crate::variables::binding::ParseError as VariableParseError;
use crate::variables::substitution::{ResolvedToken, UnresolvedVariable};

#[derive(Debug, Clone, thiserror::Error)]
pub enum LexError {
    #[error("Unclosed brace starting at position {}", span.start)]
    UnclosedBrace { span: SimpleSpan },

    #[error("Empty directive at position {}", span.start)]
    EmptyDirective { span: SimpleSpan },
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Lexer error: {0}")]
    Lex(#[from] LexError),

    #[error("Variable parsing error: {0}")]
    VariableParse(#[from] VariableParseError),

    #[error("Unresolved variable: {0}")]
    UnresolvedVariable(#[from] UnresolvedVariable),

    #[error("Parse error at position {}", span.start)]
    Parse { span: SimpleSpan, error: Rich<'static, (ResolvedToken, SimpleSpan), SimpleSpan> },
}
