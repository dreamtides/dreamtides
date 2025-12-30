use chumsky::span::SimpleSpan;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use serde::Serialize;

use crate::lexer::lexer_token::{Spanned, Token};
use crate::variables::parser_bindings::VariableBindings;

static DIRECTIVES: &[(&str, &str, VariableConstructor)] = &[
    ("a-figment", "figment", figment),
    ("a-subtype", "subtype", subtype),
    ("cards-numeral", "cards", integer),
    ("cards", "cards", integer),
    ("copies", "number", integer),
    ("count-allied-subtype", "", subtype_count),
    ("count-allies", "allies", integer),
    ("count", "count", integer),
    ("discards", "discards", integer),
    ("e", "e", integer),
    ("figments", "figment", figment),
    ("foresee", "foresee", integer),
    ("Foresee", "foresee", integer),
    ("it-or-them", "number", integer),
    ("kindle", "k", integer),
    ("Kindle", "k", integer),
    ("maximum-energy", "max", integer),
    ("mode1-cost", "mode1-cost", integer),
    ("mode2-cost", "mode2-cost", integer),
    ("MultiplyBy", "number", integer),
    ("n-figments", "", figment_count),
    ("n-random-characters", "number", integer),
    ("plural-subtype", "subtype", subtype),
    ("points", "points", integer),
    ("reclaimforcost", "reclaim", integer),
    ("s", "s", integer),
    ("subtype", "subtype", subtype),
    ("text-number", "number", integer),
    ("this-turn-times", "number", integer),
    ("top-n-cards", "to-void", integer),
    ("up-to-n-allies", "number", integer),
    ("up-to-n-events", "number", integer),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ResolvedToken {
    Token(Token),
    Integer { directive: String, value: u32 },
    Subtype { directive: String, subtype: CardSubtype },
    Figment { directive: String, figment_type: FigmentType },
    FigmentCount { directive: String, count: u32, figment_type: FigmentType },
    SubtypeCount { directive: String, count: u32, subtype: CardSubtype },
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unresolved variable: {name}")]
pub struct UnresolvedVariable {
    pub name: String,
    pub span: SimpleSpan,
}

pub fn resolve_variables(
    tokens: &[Spanned<Token>],
    bindings: &VariableBindings,
) -> Result<Vec<Spanned<ResolvedToken>>, UnresolvedVariable> {
    tokens
        .iter()
        .map(|(token, span)| match token {
            Token::Directive(name) => {
                resolve_directive(name, bindings, *span).map(|resolved| (resolved, *span))
            }
            _ => Ok((ResolvedToken::Token(token.clone()), *span)),
        })
        .collect()
}

pub fn directive_names() -> impl Iterator<Item = &'static str> {
    DIRECTIVES.iter().map(|(name, _, _)| *name)
}

pub fn variable_names() -> impl Iterator<Item = &'static str> {
    DIRECTIVES
        .iter()
        .map(|(_, var_name, _)| *var_name)
        .filter(|name| !name.is_empty())
        .chain(["figment", "number", "subtype", "allies"])
}

type VariableConstructor =
    fn(&str, &str, &VariableBindings, SimpleSpan) -> Result<ResolvedToken, UnresolvedVariable>;

fn figment_count(
    directive: &str,
    _variable_name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    let figment_type = bindings
        .get_figment("figment")
        .ok_or_else(|| UnresolvedVariable { name: "figment".to_string(), span })?;
    let count = bindings
        .get_integer("number")
        .ok_or_else(|| UnresolvedVariable { name: "number".to_string(), span })?;
    Ok(ResolvedToken::FigmentCount { directive: directive.to_string(), count, figment_type })
}

fn subtype_count(
    directive: &str,
    _variable_name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    let subtype = bindings
        .get_subtype("subtype")
        .ok_or_else(|| UnresolvedVariable { name: "subtype".to_string(), span })?;
    let count = bindings
        .get_integer("allies")
        .ok_or_else(|| UnresolvedVariable { name: "allies".to_string(), span })?;
    Ok(ResolvedToken::SubtypeCount { directive: directive.to_string(), count, subtype })
}

fn figment(
    directive: &str,
    _variable_name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    let figment_type = bindings
        .get_figment("figment")
        .ok_or_else(|| UnresolvedVariable { name: "figment".to_string(), span })?;
    Ok(ResolvedToken::Figment { directive: directive.to_string(), figment_type })
}

fn subtype(
    directive: &str,
    _variable_name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    let subtype = bindings
        .get_subtype("subtype")
        .ok_or_else(|| UnresolvedVariable { name: "subtype".to_string(), span })?;
    Ok(ResolvedToken::Subtype { directive: directive.to_string(), subtype })
}

fn integer(
    directive: &str,
    variable_name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    let value = bindings
        .get_integer(variable_name)
        .ok_or_else(|| UnresolvedVariable { name: variable_name.to_string(), span })?;
    Ok(ResolvedToken::Integer { directive: directive.to_string(), value })
}

fn resolve_directive(
    name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    match DIRECTIVES.iter().find(|(directive_name, _, _)| *directive_name == name) {
        Some((_, variable_name, constructor)) => constructor(name, variable_name, bindings, span),
        None => Ok(ResolvedToken::Token(Token::Directive(name.to_string()))),
    }
}
