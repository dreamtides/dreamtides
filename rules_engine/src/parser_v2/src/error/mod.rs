pub mod diagnostic;
pub mod recovery;

use crate::lexer::span::Span;

#[derive(Debug, Clone, thiserror::Error)]
pub enum LexError {
    #[error("Unclosed brace starting at position {}", span.start)]
    UnclosedBrace { span: Span },

    #[error("Empty directive at position {}", span.start)]
    EmptyDirective { span: Span },
}
