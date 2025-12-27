use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::{SimpleSpan, Span};

use crate::error::parser_errors::{LexError, ParserError};
use crate::error::suggestions;

pub fn format_error(error: &ParserError, source: &str, filename: &str) -> String {
    let mut output = Vec::new();

    match error {
        ParserError::Lex(lex_error) => {
            format_lex_error(lex_error, source, filename, &mut output);
        }
        ParserError::VariableParse(parse_error) => {
            Report::<(&str, std::ops::Range<usize>)>::build(ReportKind::Error, filename, 0)
                .with_message(format!("Variable parsing error: {parse_error}"))
                .finish()
                .write((filename, Source::from(source)), &mut output)
                .unwrap();
        }
        ParserError::UnresolvedVariable(unresolved) => {
            let mut report = Report::<(&str, std::ops::Range<usize>)>::build(
                ReportKind::Error,
                filename,
                unresolved.span.start(),
            )
            .with_message(format!("Unresolved variable: {}", unresolved.name));

            let mut label = Label::new((filename, unresolved.span.start()..unresolved.span.end()))
                .with_color(Color::Red);

            if let Some(suggestions) = suggestions::suggest_variable(&unresolved.name) {
                label = label.with_message(format!(
                    "Variable '{}' not found in bindings. Did you mean '{}'?",
                    unresolved.name,
                    suggestions.join("', '")
                ));
            } else {
                label = label
                    .with_message(format!("Variable '{}' not found in bindings", unresolved.name));
            }

            report = report.with_label(label);

            if let Some(suggestions) = suggestions::suggest_variable(&unresolved.name) {
                report = report
                    .with_note(format!("Available variables include: {}", suggestions.join(", ")));
            }

            report.finish().write((filename, Source::from(source)), &mut output).unwrap();
        }
        ParserError::Parse { span, error } => {
            let message = format!("Parse error: {:?}", error.reason());

            let mut report = Report::<(&str, std::ops::Range<usize>)>::build(
                ReportKind::Error,
                filename,
                span.start(),
            )
            .with_message(&message);

            let label = Label::new((filename, span.start()..span.end()))
                .with_message("Failed to parse ability text here")
                .with_color(Color::Red);

            report = report.with_label(label);

            if !error.expected().collect::<Vec<_>>().is_empty() {
                let expected_tokens: Vec<String> =
                    error.expected().map(|e| format!("{e:?}")).collect();
                report =
                    report.with_note(format!("Expected one of: {}", expected_tokens.join(", ")));
            }

            report.finish().write((filename, Source::from(source)), &mut output).unwrap();
        }
    }

    String::from_utf8(output).unwrap()
}

fn format_lex_error(error: &LexError, source: &str, filename: &str, output: &mut Vec<u8>) {
    match error {
        LexError::UnclosedBrace { span } => {
            build_lex_error_report(
                "Unclosed brace",
                "Expected closing '}'",
                *span,
                source,
                filename,
                output,
            );
        }
        LexError::EmptyDirective { span } => {
            build_lex_error_report(
                "Empty directive",
                "Directives cannot be empty",
                *span,
                source,
                filename,
                output,
            );
        }
    }
}

fn build_lex_error_report(
    message: &str,
    label: &str,
    span: SimpleSpan,
    source: &str,
    filename: &str,
    output: &mut Vec<u8>,
) {
    Report::<(&str, std::ops::Range<usize>)>::build(ReportKind::Error, filename, span.start())
        .with_message(message)
        .with_label(
            Label::new((filename, span.start()..span.end()))
                .with_message(label)
                .with_color(Color::Red),
        )
        .finish()
        .write((filename, Source::from(source)), output)
        .unwrap();
}
