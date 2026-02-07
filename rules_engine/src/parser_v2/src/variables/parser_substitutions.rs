use chumsky::span::SimpleSpan;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use serde::Serialize;

use crate::lexer::lexer_token::{Spanned, Token};
use crate::variables::parser_bindings::VariableBindings;

static DIRECTIVES: &[(&str, &str, VariableConstructor)] = &[
    ("a-figment", "figment", figment),
    ("a-subtype", "subtype", subtype),
    ("asubtype", "subtype", subtype),
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
    ("multiplyby", "number", integer),
    ("n-figments", "", figment_count),
    ("n-random-characters", "number", integer),
    ("plural-subtype", "subtype", subtype),
    ("points", "points", integer),
    ("reclaim-for-cost", "reclaim", integer),
    ("reclaimforcost", "reclaim", integer),
    ("s", "s", integer),
    ("subtype", "subtype", subtype),
    ("text-number", "number", integer),
    ("this-turn-times", "number", integer),
    ("top-n-cards", "to-void", integer),
    ("up-to-n-allies", "number", integer),
    ("up-to-n-events", "number", integer),
];

/// Maps an RLF phrase name (with optional @-transforms and :selector suffix)
/// back to the original Fluent directive name.
///
/// For most phrases, the variable argument becomes the directive name (e.g.,
/// `cards(discards)` -> directive "discards"). For phrases where the
/// directive name differs from the variable name, a special mapping is used.
static RLF_PHRASE_TO_DIRECTIVE: &[(&str, &str)] = &[
    ("cards_numeral", "cards-numeral"),
    ("top_n_cards", "top-n-cards"),
    ("count_allies", "count-allies"),
    ("count_allied_subtype", "count-allied-subtype"),
    ("a_figment", "a-figment"),
    ("n_figments", "n-figments"),
    ("this_turn_times", "this-turn-times"),
    ("n_random_characters", "n-random-characters"),
    ("up_to_n_events", "up-to-n-events"),
    ("up_to_n_allies", "up-to-n-allies"),
    ("it_or_them", "it-or-them"),
    ("text_number", "text-number"),
    ("maximum_energy", "maximum-energy"),
    ("reclaim_for_cost", "reclaim-for-cost"),
    ("Reclaim_for_cost", "reclaim-for-cost"),
    ("multiply_by", "multiplyby"),
    ("count", "count"),
];

/// Maps bare RLF reference names (without parentheses) to the directive names
/// expected by the parser.
static RLF_BARE_TO_DIRECTIVE: &[(&str, &str)] = &[
    ("energy_symbol", "energy-symbol"),
    ("choose_one", "chooseone"),
    ("judgment_phase_name", "judgmentphasename"),
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

fn resolve_directive(
    name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    // Try direct lookup in DIRECTIVES first (handles legacy Fluent syntax)
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

    // Try bare RLF reference mapping (e.g., energy_symbol -> energy-symbol)
    if let Some((_, directive_name)) =
        RLF_BARE_TO_DIRECTIVE.iter().find(|(rlf_name, _)| *rlf_name == name)
    {
        return Ok(ResolvedToken::Token(Token::Directive(directive_name.to_string())));
    }

    Ok(ResolvedToken::Token(Token::Directive(name.to_string())))
}

/// Resolves an RLF function call directive like `energy(e)`,
/// `@a subtype(subtype)`, or `subtype(subtype):other` by mapping back to
/// the original Fluent directive name and resolving via the DIRECTIVES table.
fn resolve_rlf_directive(
    name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<Option<ResolvedToken>, UnresolvedVariable> {
    // Check for @-transforms to determine the original directive prefix
    let has_cap = name.starts_with("@cap ");
    let stripped = name.trim_start_matches("@cap ").trim_start_matches("@a ");

    // Check for :selector suffix (e.g., subtype(subtype):other -> plural-subtype)
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

    // Determine the Fluent directive name from the RLF phrase syntax.
    // Convert underscores to hyphens to match DIRECTIVES table naming.
    let directive_name =
        determine_fluent_directive(name, phrase_name, &args, has_cap, selector).replace('_', "-");

    // Look up the directive in the DIRECTIVES table
    if let Some((_, default_var, constructor)) =
        DIRECTIVES.iter().find(|(dir_name, _, _)| *dir_name == directive_name)
    {
        // Use the RLF argument as the variable name for lookups when present,
        // otherwise fall back to the default variable name from DIRECTIVES.
        // Convert underscores to hyphens since binding keys use hyphenated
        // names from the TOML card definitions.
        let variable_name = if args.len() == 1 && !args[0].is_empty() {
            args[0].replace('_', "-")
        } else {
            default_var.to_string()
        };
        return Ok(Some(constructor(&directive_name, &variable_name, bindings, span)?));
    }

    // Try numbered variant for the first argument (e.g., cards(cards1))
    if !args.is_empty() {
        let arg = args[0];
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

/// Determines the original Fluent directive name from RLF phrase syntax.
fn determine_fluent_directive(
    full_name: &str,
    phrase_name: &str,
    args: &[&str],
    has_cap: bool,
    selector: Option<&str>,
) -> String {
    // Check if the full name starts with @a or @cap @a
    let has_a = full_name.contains("@a ");

    // Handle subtype-related directives based on @-transforms and selectors
    if phrase_name == "subtype" {
        if has_a && has_cap {
            return "asubtype".to_string();
        }
        if has_a {
            return "a-subtype".to_string();
        }
        if selector == Some("other") {
            return "plural-subtype".to_string();
        }
        return "subtype".to_string();
    }

    // Check the special phrase-to-directive mapping table
    if let Some((_, dir_name)) =
        RLF_PHRASE_TO_DIRECTIVE.iter().find(|(rlf_name, _)| *rlf_name == phrase_name)
    {
        return dir_name.to_string();
    }

    // For phrases where the phrase name itself is the directive (not the
    // argument), return the phrase name. These are directives where the parser
    // matches on the directive name rather than the variable name.
    if matches!(phrase_name, "kindle" | "reclaimforcost" | "multiplyby") {
        return phrase_name.to_string();
    }

    // For simple RLF phrases like energy(e), cards(cards), points(points),
    // cards(discards), etc.: the variable argument IS the original Fluent
    // directive name
    if args.len() == 1 {
        return args[0].to_string();
    }

    // Fallback: use the phrase name as-is
    phrase_name.to_string()
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
