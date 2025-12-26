use ability_data::figment_type::FigmentType;
use ability_data::variable_value::VariableValue;
use chumsky::span::SimpleSpan;
use core_data::card_types::CardSubtype;
use serde::Serialize;

use crate::lexer::token::{Spanned, Token};
use crate::variables::binding::VariableBindings;

pub static COMPOUND_DIRECTIVES: &[&str] = &["n-figments", "a-figment"];

pub static VARIABLE_DIRECTIVES: &[&str] = &[
    "e",
    "e-",
    "e_",
    "cards",
    "cards-v2",
    "discards",
    "points",
    "s",
    "count",
    "k",
    "k2",
    "subtype",
    "plural-subtype",
    "a-subtype",
    "count-allies",
    "count-allied-subtype",
    "cards-numeral",
    "energy-symbol",
    "top-n-cards",
    "up-to-n-allies",
    "up-to-n-events",
    "n-random-characters",
    "it-or-them",
    "this-turn-times",
    "maximum-energy",
    "reclaim-for-cost",
    "text-number",
    "number",
    "figment",
    "figments",
    "allies",
    "to-void",
    "foresee",
    "max",
    "MultiplyBy",
    "copies",
];

pub static DIRECTIVE_VARIABLE_MAPPINGS: &[(&str, &str)] = &[
    ("a-subtype", "subtype"),
    ("cards-numeral", "cards"),
    ("copies", "number"),
    ("count-allied-subtype", "allies"),
    ("count-allies", "allies"),
    ("e-", "e_"),
    ("figments", "figment"),
    ("it-or-them", "number"),
    ("kindle", "k"),
    ("kindle-k2", "k2"),
    ("maximum-energy", "max"),
    ("MultiplyBy", "number"),
    ("n-random-characters", "number"),
    ("plural-subtype", "subtype"),
    ("reclaim-for-cost", "reclaim"),
    ("ReclaimForCost", "reclaim"),
    ("text-number", "number"),
    ("this-turn-times", "number"),
    ("top-n-cards", "to-void"),
    ("up-to-n-allies", "number"),
    ("up-to-n-events", "number"),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ResolvedToken {
    Token(Token),
    Integer { directive: String, value: u32 },
    Subtype { directive: String, subtype: CardSubtype },
    FigmentCount { count: u32, figment_type: FigmentType },
    FigmentSingle { figment_type: FigmentType },
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
            Token::Directive(name) if is_compound_directive(name) => {
                resolve_compound_directive(name, bindings, *span).map(|resolved| (resolved, *span))
            }
            Token::Directive(name) if is_variable_directive(name) => {
                let var_name = extract_variable_name(name);
                match bindings.get(&var_name) {
                    Some(VariableValue::Integer(n)) => {
                        Ok((ResolvedToken::Integer { directive: name.clone(), value: *n }, *span))
                    }
                    Some(VariableValue::Subtype(s)) => {
                        Ok((ResolvedToken::Subtype { directive: name.clone(), subtype: *s }, *span))
                    }
                    Some(VariableValue::Figment(_)) => {
                        Err(UnresolvedVariable { name: var_name.clone(), span: *span })
                    }
                    None => Err(UnresolvedVariable { name: var_name.clone(), span: *span }),
                }
            }
            _ => Ok((ResolvedToken::Token(token.clone()), *span)),
        })
        .collect()
}

fn is_compound_directive(name: &str) -> bool {
    COMPOUND_DIRECTIVES.contains(&name)
}

fn is_variable_directive(name: &str) -> bool {
    VARIABLE_DIRECTIVES.contains(&name)
}

fn extract_variable_name(directive: &str) -> String {
    DIRECTIVE_VARIABLE_MAPPINGS
        .iter()
        .find(|(from, _)| *from == directive)
        .map(|(_, to)| (*to).to_string())
        .unwrap_or_else(|| directive.to_string())
}

fn resolve_compound_directive(
    name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    match name {
        "n-figments" => {
            let count = bindings
                .get_integer("number")
                .ok_or_else(|| UnresolvedVariable { name: "number".to_string(), span })?;
            let figment_type = bindings
                .get_figment("figment")
                .ok_or_else(|| UnresolvedVariable { name: "figment".to_string(), span })?;
            Ok(ResolvedToken::FigmentCount { count, figment_type })
        }
        "a-figment" => {
            let figment_type = bindings
                .get_figment("figment")
                .ok_or_else(|| UnresolvedVariable { name: "figment".to_string(), span })?;
            Ok(ResolvedToken::FigmentSingle { figment_type })
        }
        _ => Err(UnresolvedVariable { name: name.to_string(), span }),
    }
}
