use chumsky::span::{SimpleSpan, Span};
use parser_v2::error::parser_errors::LexError;
use parser_v2::lexer::lexer_token::Token;
use parser_v2::lexer::lexer_tokenize::{self, LexResult};

fn tokens(result: &LexResult) -> Vec<&Token> {
    result.tokens.iter().map(|(t, _)| t).collect()
}

fn spans(result: &LexResult) -> Vec<SimpleSpan> {
    result.tokens.iter().map(|(_, s)| *s).collect()
}

#[test]
fn test_card_1_play_card_count() {
    let input = "When you play {$c} {card:$c} in a turn, {reclaim} this character.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Word("when".to_string()),
        &Token::Word("you".to_string()),
        &Token::Word("play".to_string()),
        &Token::Directive("$c".to_string()),
        &Token::Directive("card:$c".to_string()),
        &Token::Word("in".to_string()),
        &Token::Word("a".to_string()),
        &Token::Word("turn".to_string()),
        &Token::Comma,
        &Token::Directive("reclaim".to_string()),
        &Token::Word("this".to_string()),
        &Token::Word("character".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_card_2_discover() {
    let input = "{Discover} a card with cost {e}.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Directive("discover".to_string()),
        &Token::Word("a".to_string()),
        &Token::Word("card".to_string()),
        &Token::Word("with".to_string()),
        &Token::Word("cost".to_string()),
        &Token::Directive("e".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_card_3_judgment_return() {
    let input = "{Judgment} Return this character from your void to your hand.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Directive("judgment".to_string()),
        &Token::Word("return".to_string()),
        &Token::Word("this".to_string()),
        &Token::Word("character".to_string()),
        &Token::Word("from".to_string()),
        &Token::Word("your".to_string()),
        &Token::Word("void".to_string()),
        &Token::Word("to".to_string()),
        &Token::Word("your".to_string()),
        &Token::Word("hand".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_card_4_judgment_may_discard() {
    let input = "{Judgment} You may discard {discards} to draw {cards} and gain {points}.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Directive("judgment".to_string()),
        &Token::Word("you".to_string()),
        &Token::Word("may".to_string()),
        &Token::Word("discard".to_string()),
        &Token::Directive("discards".to_string()),
        &Token::Word("to".to_string()),
        &Token::Word("draw".to_string()),
        &Token::Directive("cards".to_string()),
        &Token::Word("and".to_string()),
        &Token::Word("gain".to_string()),
        &Token::Directive("points".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_card_5_draw_discard_reclaim() {
    let input = "Draw {cards}. Discard {discards}.\n\n{Reclaim_For_Cost}";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Word("draw".to_string()),
        &Token::Directive("cards".to_string()),
        &Token::Period,
        &Token::Word("discard".to_string()),
        &Token::Directive("discards".to_string()),
        &Token::Period,
        &Token::Newline,
        &Token::Newline,
        &Token::Directive("reclaim_for_cost".to_string()),
    ]);
}

#[test]
fn test_card_6_judgment_banish_dissolve() {
    let input =
        "{Judgment} You may {banish} {cards} from your void to {dissolve} an enemy with cost {e} or less.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Directive("judgment".to_string()),
        &Token::Word("you".to_string()),
        &Token::Word("may".to_string()),
        &Token::Directive("banish".to_string()),
        &Token::Directive("cards".to_string()),
        &Token::Word("from".to_string()),
        &Token::Word("your".to_string()),
        &Token::Word("void".to_string()),
        &Token::Word("to".to_string()),
        &Token::Directive("dissolve".to_string()),
        &Token::Word("an".to_string()),
        &Token::Word("enemy".to_string()),
        &Token::Word("with".to_string()),
        &Token::Word("cost".to_string()),
        &Token::Directive("e".to_string()),
        &Token::Word("or".to_string()),
        &Token::Word("less".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_card_7_once_per_turn_play() {
    let input = "Once per turn, you may play a character with cost {e} or less from your void.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Word("once".to_string()),
        &Token::Word("per".to_string()),
        &Token::Word("turn".to_string()),
        &Token::Comma,
        &Token::Word("you".to_string()),
        &Token::Word("may".to_string()),
        &Token::Word("play".to_string()),
        &Token::Word("a".to_string()),
        &Token::Word("character".to_string()),
        &Token::Word("with".to_string()),
        &Token::Word("cost".to_string()),
        &Token::Directive("e".to_string()),
        &Token::Word("or".to_string()),
        &Token::Word("less".to_string()),
        &Token::Word("from".to_string()),
        &Token::Word("your".to_string()),
        &Token::Word("void".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_card_8_when_discard_kindle() {
    let input = "When you discard a card, {kindle}.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Word("when".to_string()),
        &Token::Word("you".to_string()),
        &Token::Word("discard".to_string()),
        &Token::Word("a".to_string()),
        &Token::Word("card".to_string()),
        &Token::Comma,
        &Token::Directive("kindle".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_card_9_materialize_figments() {
    let input = "{Materialize} {n_figments}.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Directive("materialize".to_string()),
        &Token::Directive("n_figments".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_span_tracking() {
    let input = "Draw {cards}.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(spans(&result), vec![
        SimpleSpan::new((), 0..4),
        SimpleSpan::new((), 5..12),
        SimpleSpan::new((), 12..13),
    ]);
}

#[test]
fn test_colon_token() {
    let input = "{e}: Draw {cards}.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Directive("e".to_string()),
        &Token::Colon,
        &Token::Word("draw".to_string()),
        &Token::Directive("cards".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_numeric_words() {
    let input = "Draw 2 cards.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Word("draw".to_string()),
        &Token::Word("2".to_string()),
        &Token::Word("cards".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_plus_symbol_as_word() {
    let input = "+2 spark.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Word("+2".to_string()),
        &Token::Word("spark".to_string()),
        &Token::Period,
    ]);
}

#[test]
fn test_lowercase_conversion() {
    let input = "DRAW Cards.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Word("draw".to_string()),
        &Token::Word("cards".to_string()),
        &Token::Period,
    ]);
    assert_eq!(result.original, "DRAW Cards.");
}

#[test]
fn test_directive_lowercase_conversion() {
    let input = "{Judgment} {CARDS}";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Directive("judgment".to_string()),
        &Token::Directive("cards".to_string()),
    ]);
}

#[test]
fn test_empty_input() {
    let input = "";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert!(result.tokens.is_empty());
}

#[test]
fn test_whitespace_only() {
    let input = "   \t  ";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert!(result.tokens.is_empty());
}

#[test]
fn test_error_unclosed_brace() {
    let input = "{unclosed";
    let result = lexer_tokenize::lex(input);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, LexError::UnclosedBrace { .. }));
}

#[test]
fn test_error_empty_directive() {
    let input = "{}";
    let result = lexer_tokenize::lex(input);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, LexError::EmptyDirective { .. }));
}

#[test]
fn test_multiple_newlines() {
    let input = "a\n\nb";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Word("a".to_string()),
        &Token::Newline,
        &Token::Newline,
        &Token::Word("b".to_string()),
    ]);
}

#[test]
fn test_hyphenated_words() {
    let input = "once-per-turn";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![&Token::Word("once-per-turn".to_string()),]);
}

#[test]
fn test_apostrophe_in_word() {
    let input = "opponent's";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![&Token::Word("opponent's".to_string()),]);
}

#[test]
fn test_combined_trigger_directive() {
    let input = "{Materialized_Judgment} Gain {e}.";
    let result = lexer_tokenize::lex(input).expect("lexing should succeed");

    assert_eq!(tokens(&result), vec![
        &Token::Directive("materialized_judgment".to_string()),
        &Token::Word("gain".to_string()),
        &Token::Directive("e".to_string()),
        &Token::Period,
    ]);
}
