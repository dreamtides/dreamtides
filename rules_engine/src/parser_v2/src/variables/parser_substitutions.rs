use chumsky::span::SimpleSpan;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use serde::Serialize;

use crate::lexer::lexer_token::{Spanned, Token};
use crate::variables::parser_bindings::VariableBindings;

static DIRECTIVES: &[(&str, &str, VariableConstructor)] = &[
    ("a_figment", "figment", figment),
    ("a_subtype", "subtype", subtype),
    ("asubtype", "subtype", subtype),
    ("cards", "cards", integer),
    ("cards_numeral", "cards", integer),
    ("choose_one", "", bare),
    ("copies", "number", integer),
    ("count", "count", integer),
    ("count_allied_subtype", "", subtype_count),
    ("count_allies", "allies", integer),
    ("discards", "discards", integer),
    ("e", "e", integer),
    ("energy_symbol", "", bare),
    ("figments", "figment", figment),
    ("foresee", "foresee", integer),
    ("Foresee", "foresee", integer),
    ("it_or_them", "number", integer),
    ("judgment_phase_name", "", bare),
    ("kindle", "k", integer),
    ("Kindle", "k", integer),
    ("maximum_energy", "max", integer),
    ("mode1_cost", "mode1_cost", integer),
    ("mode2_cost", "mode2_cost", integer),
    ("multiply_by", "number", integer),
    ("n_figments", "", figment_count),
    ("n_random_characters", "number", integer),
    ("plural_subtype", "subtype", subtype),
    ("points", "points", integer),
    ("reclaim_for_cost", "reclaim", integer),
    ("reclaimforcost", "reclaim", integer),
    ("s", "s", integer),
    ("subtype", "subtype", subtype),
    ("text_number", "number", integer),
    ("this_turn_times", "number", integer),
    ("top_n_cards", "to_void", integer),
    ("up_to_n_allies", "number", integer),
    ("up_to_n_events", "number", integer),
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

/// Returns the integer variable name for a given directive, if it exists and is
/// an integer type. Returns None for subtypes, figments, compound directives,
/// or unknown directives.
pub fn directive_to_integer_variable(directive: &str) -> Option<&'static str> {
    DIRECTIVES
        .iter()
        .find(|(dir_name, _, constructor)| {
            *dir_name == directive && std::ptr::eq(*constructor as *const (), integer as *const ())
        })
        .and_then(|(_, var_name, _)| if var_name.is_empty() { None } else { Some(*var_name) })
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

fn bare(
    directive: &str,
    _variable_name: &str,
    _bindings: &VariableBindings,
    _span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    Ok(ResolvedToken::Token(Token::Directive(directive.to_string())))
}

fn resolve_directive(
    name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    // Try direct lookup in DIRECTIVES first
    if let Some((_, variable_name, constructor)) =
        DIRECTIVES.iter().find(|(directive_name, _, _)| *directive_name == name)
    {
        return constructor(name, variable_name, bindings, span);
    }

    // Try numbered variant lookup (e.g., cards1, e2)
    if let Some((base_name, _suffix)) = split_numbered_directive(name) {
        if let Some((_, _, constructor)) =
            DIRECTIVES.iter().find(|(directive_name, _, _)| *directive_name == base_name)
        {
            return constructor(name, name, bindings, span);
        }
    }

    // Try RLF function call syntax: phrase(args) or phrase(args):selector
    if let Some(resolved) = resolve_rlf_directive(name, bindings, span)? {
        return Ok(resolved);
    }

    Ok(ResolvedToken::Token(Token::Directive(name.to_string())))
}

/// Resolves an RLF function call directive like `energy(e)`,
/// `@a subtype(subtype)`, or `subtype(subtype):other` by looking up the
/// phrase name or argument in the DIRECTIVES table.
fn resolve_rlf_directive(
    name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<Option<ResolvedToken>, UnresolvedVariable> {
    let has_cap = name.starts_with("@cap ");
    let has_a = name.contains("@a ");
    let stripped = name.trim_start_matches("@cap ").trim_start_matches("@a ");

    // Check for :selector suffix (e.g., subtype(subtype):other -> plural_subtype)
    let (core, selector) = if let Some(pos) = stripped.find(':') {
        (&stripped[..pos], Some(&stripped[pos + 1..]))
    } else {
        (stripped, None)
    };

    // Parse phrase_name(args)
    let Some(paren_start) = core.find('(') else {
        return Ok(None);
    };
    let Some(paren_end) = core.find(')') else {
        return Ok(None);
    };

    let phrase_name = core[..paren_start].trim();
    let args_str = &core[paren_start + 1..paren_end];
    let args: Vec<&str> = args_str.split(',').map(str::trim).collect();

    // Handle subtype-related directives based on @-transforms and selectors
    if phrase_name == "subtype" {
        let directive_name = if has_a && has_cap {
            "asubtype"
        } else if has_a {
            "a_subtype"
        } else if selector == Some("other") {
            "plural_subtype"
        } else {
            "subtype"
        };
        if let Some((_, default_var, constructor)) =
            DIRECTIVES.iter().find(|(dir_name, _, _)| *dir_name == directive_name)
        {
            let variable_name =
                if args.len() == 1 && !args[0].is_empty() { args[0] } else { default_var };
            return Ok(Some(constructor(directive_name, variable_name, bindings, span)?));
        }
    }

    // For phrases whose name is itself a distinct directive (e.g.,
    // cards_numeral, top_n_cards, count_allies), the phrase name is the
    // directive and the argument is the variable name.
    if let Some((_, default_var, constructor)) =
        DIRECTIVES.iter().find(|(dir_name, _, _)| *dir_name == phrase_name)
    {
        if args.len() == 1 && !args[0].is_empty() {
            let arg = args[0];
            // If the argument is also a directive name, resolve as the
            // argument directive instead (e.g., cards(discards) → directive
            // "discards"). This handles simple wrapper phrases where the
            // argument is the actual directive.
            if arg != phrase_name {
                if let Some((arg_dir, _, arg_constructor)) =
                    DIRECTIVES.iter().find(|(dir_name, _, _)| *dir_name == arg)
                {
                    // Argument is itself a directive — check if the phrase's
                    // default variable matches the argument. If so, the phrase
                    // IS the directive (e.g., cards_numeral(cards) where
                    // default var is "cards"). If not, the argument is the
                    // directive (e.g., cards(discards) where default var is
                    // "cards" but arg is "discards").
                    if *default_var != arg {
                        return Ok(Some(arg_constructor(arg_dir, arg, bindings, span)?));
                    }
                }
            }
            return Ok(Some(constructor(phrase_name, arg, bindings, span)?));
        }
        return Ok(Some(constructor(phrase_name, default_var, bindings, span)?));
    }

    // Fall back to argument as directive name.
    if args.len() == 1 {
        let arg = args[0];
        if let Some((_, default_var, constructor)) =
            DIRECTIVES.iter().find(|(dir_name, _, _)| *dir_name == arg)
        {
            return Ok(Some(constructor(arg, default_var, bindings, span)?));
        }

        // Try numbered variant for the argument (e.g., cards(cards1))
        if let Some((base_name, _suffix)) = split_numbered_directive(arg) {
            if let Some((_, _, constructor)) =
                DIRECTIVES.iter().find(|(dir_name, _, _)| *dir_name == base_name)
            {
                return Ok(Some(constructor(arg, arg, bindings, span)?));
            }
        }
    }

    Ok(None)
}

fn split_numbered_directive(name: &str) -> Option<(&str, &str)> {
    let mut split_pos = name.len();
    while split_pos > 0 && name.as_bytes()[split_pos - 1].is_ascii_digit() {
        split_pos -= 1;
    }

    if split_pos < name.len() && split_pos > 0 {
        Some((&name[..split_pos], &name[split_pos..]))
    } else {
        None
    }
}
