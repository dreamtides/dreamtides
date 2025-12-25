use std::iter::Peekable;
use std::str::CharIndices;

use crate::error::LexError;
use crate::lexer::span::Span;
use crate::lexer::token::{Spanned, Token};

#[derive(Debug, Clone)]
pub struct LexResult {
    pub tokens: Vec<Spanned<Token>>,
    pub original: String,
}

pub fn lex(input: &str) -> Result<LexResult, LexError> {
    let original = input.to_string();
    let lowercased = input.to_lowercase();
    let mut tokens = Vec::new();
    let mut chars = lowercased.char_indices().peekable();

    while let Some((start, ch)) = chars.next() {
        match ch {
            '{' => tokens.push(lex_directive(&mut chars, start)?),
            '.' => tokens.push((Token::Period, Span::new(start, start + 1))),
            ',' => tokens.push((Token::Comma, Span::new(start, start + 1))),
            ':' => tokens.push((Token::Colon, Span::new(start, start + 1))),
            '\n' => tokens.push((Token::Newline, Span::new(start, start + 1))),
            c if c.is_whitespace() => {}
            _ => tokens.push(lex_word(&mut chars, start, ch)),
        }
    }

    Ok(LexResult { tokens, original })
}

fn lex_directive(
    chars: &mut Peekable<CharIndices<'_>>,
    start: usize,
) -> Result<Spanned<Token>, LexError> {
    let mut name = String::new();
    let content_start = start + 1;

    loop {
        match chars.next() {
            Some((end, '}')) => {
                if name.is_empty() {
                    return Err(LexError::EmptyDirective { span: Span::new(start, end + 1) });
                }
                return Ok((Token::Directive(name), Span::new(start, end + 1)));
            }
            Some((_, ch)) => {
                name.push(ch);
            }
            None => {
                return Err(LexError::UnclosedBrace {
                    span: Span::new(start, content_start + name.len()),
                });
            }
        }
    }
}

fn lex_word(
    chars: &mut Peekable<CharIndices<'_>>,
    start: usize,
    first_char: char,
) -> Spanned<Token> {
    let mut word = String::new();
    word.push(first_char);
    let mut end = start + first_char.len_utf8();

    while let Some(&(pos, ch)) = chars.peek() {
        if is_word_char(ch) {
            word.push(ch);
            end = pos + ch.len_utf8();
            chars.next();
        } else {
            break;
        }
    }

    (Token::Word(word), Span::new(start, end))
}

fn is_word_char(ch: char) -> bool {
    !ch.is_whitespace() && !matches!(ch, '{' | '}' | '.' | ',' | ':' | '\n')
}
