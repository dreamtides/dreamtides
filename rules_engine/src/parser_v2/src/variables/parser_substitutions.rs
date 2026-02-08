use chumsky::span::SimpleSpan;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use serde::Serialize;

use crate::lexer::lexer_token::{Spanned, Token};
use crate::variables::parser_bindings::VariableBindings;

static BARE_PHRASES: &[&str] = &["choose_one", "energy_symbol", "judgment_phase_name"];

static FIGMENT_PHRASES: &[(&str, &str)] = &[("figment", "g"), ("figments", "g")];

static PHRASES: &[PhraseEntry] = &[
    ("cards", "c", ResolvedToken::CardCount),
    ("cards_numeral", "c", ResolvedToken::CardCountNumeral),
    ("copies", "n", ResolvedToken::Copies),
    ("count", "n", ResolvedToken::Count),
    ("count_allies", "a", ResolvedToken::CountAllies),
    ("discards", "d", ResolvedToken::DiscardCount),
    ("energy", "e", ResolvedToken::Energy),
    ("foresee", "f", ResolvedToken::ForeseeCount),
    ("it_or_them", "n", ResolvedToken::ItOrThemCount),
    ("kindle", "k", ResolvedToken::KindleAmount),
    ("maximum_energy", "m", ResolvedToken::MaximumEnergy),
    ("multiply_by", "n", ResolvedToken::Number),
    ("n_random_characters", "n", ResolvedToken::Number),
    ("points", "p", ResolvedToken::PointCount),
    ("reclaim_for_cost", "r", ResolvedToken::ReclaimCost),
    ("spark", "s", ResolvedToken::SparkAmount),
    ("text_number", "n", ResolvedToken::TextNumber),
    ("this_turn_times", "n", ResolvedToken::ThisTurnTimes),
    ("top_n_cards", "v", ResolvedToken::TopNCards),
    ("up_to_n_allies", "n", ResolvedToken::UpToNAllies),
    ("up_to_n_events", "n", ResolvedToken::UpToNEvents),
    // Short-form entries used in TOML data
    ("e", "e", ResolvedToken::Energy),
    ("s", "s", ResolvedToken::SparkAmount),
];

static SUBTYPE_PHRASES: &[(&str, &str)] =
    &[("a_subtype", "t"), ("asubtype", "t"), ("plural_subtype", "t"), ("subtype", "t")];

/// A resolved token produced by variable substitution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ResolvedToken {
    Token(Token),
    // Integer-valued semantic tokens
    Energy(u32),
    Mode1Energy(u32),
    Mode2Energy(u32),
    CardCount(u32),
    CardCountNumeral(u32),
    TopNCards(u32),
    DiscardCount(u32),
    PointCount(u32),
    SparkAmount(u32),
    ForeseeCount(u32),
    KindleAmount(u32),
    MaximumEnergy(u32),
    Count(u32),
    CountAllies(u32),
    UpToNAllies(u32),
    UpToNEvents(u32),
    ItOrThemCount(u32),
    Number(u32),
    ReclaimCost(u32),
    ThisTurnTimes(u32),
    Copies(u32),
    TextNumber(u32),
    // Subtype/figment tokens
    Subtype(CardSubtype),
    Figment(FigmentType),
    FigmentCount { count: u32, figment_type: FigmentType },
    SubtypeCount { count: u32, subtype: CardSubtype },
}

/// Error returned when a variable binding cannot be found.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Unresolved variable: {name}")]
pub struct UnresolvedVariable {
    pub name: String,
    pub span: SimpleSpan,
}

/// Resolves directive tokens using variable bindings.
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

/// Returns all phrase names for error suggestion matching.
pub fn phrase_names() -> impl Iterator<Item = &'static str> {
    PHRASES
        .iter()
        .map(|(name, _, _)| *name)
        .chain(BARE_PHRASES.iter().copied())
        .chain(SUBTYPE_PHRASES.iter().map(|(name, _)| *name))
        .chain(FIGMENT_PHRASES.iter().map(|(name, _)| *name))
        .chain(std::iter::once("n_figments"))
        .chain(std::iter::once("count_allied_subtype"))
}

/// Returns all variable names for error suggestion matching.
pub fn variable_names() -> impl Iterator<Item = &'static str> {
    PHRASES
        .iter()
        .map(|(_, var_name, _)| *var_name)
        .filter(|name| !name.is_empty())
        .chain(SUBTYPE_PHRASES.iter().map(|(_, var_name)| *var_name))
        .chain(FIGMENT_PHRASES.iter().map(|(_, var_name)| *var_name))
        .chain(["g", "n", "t", "a", "e1", "e2"])
}

type PhraseEntry = (&'static str, &'static str, fn(u32) -> ResolvedToken);

fn resolve_directive(
    name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<ResolvedToken, UnresolvedVariable> {
    // Strip $ prefix from bare variable references (e.g., "$s" -> "s")
    if let Some(bare_name) = name.strip_prefix('$') {
        return resolve_directive(bare_name, bindings, span);
    }

    // Strip RLF transform prefixes (@a, @cap, @plural) from bare references
    for prefix in ["@cap @a ", "@cap ", "@a ", "@plural "] {
        if let Some(stripped) = name.strip_prefix(prefix) {
            return resolve_directive(stripped, bindings, span);
        }
    }

    // Try direct lookup in integer phrases
    if let Some((_, default_var, constructor)) =
        PHRASES.iter().find(|(phrase_name, _, _)| *phrase_name == name)
    {
        let value = bindings
            .get_integer(default_var)
            .ok_or_else(|| UnresolvedVariable { name: default_var.to_string(), span })?;
        return Ok(constructor(value));
    }

    // Try bare phrases
    if BARE_PHRASES.contains(&name) {
        return Ok(ResolvedToken::Token(Token::Directive(name.to_string())));
    }

    // Try subtype phrases
    if let Some((_, default_var)) =
        SUBTYPE_PHRASES.iter().find(|(phrase_name, _)| *phrase_name == name)
    {
        let subtype = bindings
            .get_subtype(default_var)
            .ok_or_else(|| UnresolvedVariable { name: default_var.to_string(), span })?;
        return Ok(ResolvedToken::Subtype(subtype));
    }

    // Try figment phrases
    if let Some((_, default_var)) =
        FIGMENT_PHRASES.iter().find(|(phrase_name, _)| *phrase_name == name)
    {
        let figment_type = bindings
            .get_figment(default_var)
            .ok_or_else(|| UnresolvedVariable { name: default_var.to_string(), span })?;
        return Ok(ResolvedToken::Figment(figment_type));
    }

    // Compound: n_figments
    if name == "n_figments" {
        let figment_type = bindings
            .get_figment("g")
            .ok_or_else(|| UnresolvedVariable { name: "g".to_string(), span })?;
        let count = bindings
            .get_integer("n")
            .ok_or_else(|| UnresolvedVariable { name: "n".to_string(), span })?;
        return Ok(ResolvedToken::FigmentCount { count, figment_type });
    }

    // Compound: count_allied_subtype
    if name == "count_allied_subtype" {
        let subtype = bindings
            .get_subtype("t")
            .ok_or_else(|| UnresolvedVariable { name: "t".to_string(), span })?;
        let count = bindings
            .get_integer("a")
            .ok_or_else(|| UnresolvedVariable { name: "a".to_string(), span })?;
        return Ok(ResolvedToken::SubtypeCount { count, subtype });
    }

    // Try numbered variant lookup (e.g., e1, e2, c1, c2)
    if let Some((base, suffix)) = split_numbered_suffix(name) {
        // Energy numbered variants map to mode-specific tokens
        if base == "e" {
            let value = bindings
                .get_integer(name)
                .ok_or_else(|| UnresolvedVariable { name: name.to_string(), span })?;
            return match suffix {
                "1" => Ok(ResolvedToken::Mode1Energy(value)),
                "2" => Ok(ResolvedToken::Mode2Energy(value)),
                _ => Ok(ResolvedToken::Energy(value)),
            };
        }
        // Other numbered variants: look up the base phrase, read from full name binding
        if let Some((_, _, constructor)) =
            PHRASES.iter().find(|(phrase_name, _, _)| *phrase_name == base)
        {
            let value = bindings
                .get_integer(name)
                .ok_or_else(|| UnresolvedVariable { name: name.to_string(), span })?;
            return Ok(constructor(value));
        }
    }

    // Try RLF function call syntax: phrase(args) or phrase(args):selector
    if let Some(resolved) = resolve_rlf_syntax(name, bindings, span)? {
        return Ok(resolved);
    }

    // Unknown directive — pass through as token
    Ok(ResolvedToken::Token(Token::Directive(name.to_string())))
}

fn resolve_rlf_syntax(
    name: &str,
    bindings: &VariableBindings,
    span: SimpleSpan,
) -> Result<Option<ResolvedToken>, UnresolvedVariable> {
    // Strip @cap, @a, and @plural prefixes
    let stripped =
        name.trim_start_matches("@cap ").trim_start_matches("@a ").trim_start_matches("@plural ");

    // Strip :selector suffix
    let (core, _selector) = if let Some(pos) = stripped.find(':') {
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
    let args: Vec<&str> = args_str
        .split(',')
        .map(str::trim)
        .map(|arg| arg.strip_prefix('$').unwrap_or(arg))
        .collect();

    // Handle subtype phrases
    if phrase_name == "subtype" {
        let variable_name = if args.len() == 1 && !args[0].is_empty() { args[0] } else { "t" };
        let subtype = bindings
            .get_subtype(variable_name)
            .ok_or_else(|| UnresolvedVariable { name: variable_name.to_string(), span })?;
        return Ok(Some(ResolvedToken::Subtype(subtype)));
    }

    // Handle figment phrases
    if phrase_name == "figment" || phrase_name == "figments" {
        let variable_name = if args.len() == 1 && !args[0].is_empty() { args[0] } else { "g" };
        let figment_type = bindings
            .get_figment(variable_name)
            .ok_or_else(|| UnresolvedVariable { name: variable_name.to_string(), span })?;
        return Ok(Some(ResolvedToken::Figment(figment_type)));
    }

    // Handle n_figments compound
    if phrase_name == "n_figments" {
        let figment_type = bindings
            .get_figment("g")
            .ok_or_else(|| UnresolvedVariable { name: "g".to_string(), span })?;
        let count_var = if !args.is_empty() && !args[0].is_empty() { args[0] } else { "n" };
        let count = bindings
            .get_integer(count_var)
            .ok_or_else(|| UnresolvedVariable { name: count_var.to_string(), span })?;
        return Ok(Some(ResolvedToken::FigmentCount { count, figment_type }));
    }

    // Handle count_allied_subtype compound
    if phrase_name == "count_allied_subtype" {
        let subtype = bindings
            .get_subtype("t")
            .ok_or_else(|| UnresolvedVariable { name: "t".to_string(), span })?;
        let count_var = if !args.is_empty() && !args[0].is_empty() { args[0] } else { "a" };
        let count = bindings
            .get_integer(count_var)
            .ok_or_else(|| UnresolvedVariable { name: count_var.to_string(), span })?;
        return Ok(Some(ResolvedToken::SubtypeCount { count, subtype }));
    }

    // Handle integer phrases
    if let Some((_, default_var, constructor)) =
        PHRASES.iter().find(|(pn, _, _)| *pn == phrase_name)
    {
        let variable_name =
            if args.len() == 1 && !args[0].is_empty() { args[0] } else { default_var };

        // Handle numbered variable names (e.g., energy(e1) -> Mode1Energy)
        if let Some((base, suffix)) = split_numbered_suffix(variable_name) {
            if base == "e" && (phrase_name == "energy" || phrase_name == "e") {
                let value = bindings
                    .get_integer(variable_name)
                    .ok_or_else(|| UnresolvedVariable { name: variable_name.to_string(), span })?;
                return match suffix {
                    "1" => Ok(Some(ResolvedToken::Mode1Energy(value))),
                    "2" => Ok(Some(ResolvedToken::Mode2Energy(value))),
                    _ => Ok(Some(ResolvedToken::Energy(value))),
                };
            }
            // For other phrases with numbered vars, read from the full variable name
            let value = bindings
                .get_integer(variable_name)
                .ok_or_else(|| UnresolvedVariable { name: variable_name.to_string(), span })?;
            return Ok(Some(constructor(value)));
        }

        let value = bindings
            .get_integer(variable_name)
            .ok_or_else(|| UnresolvedVariable { name: variable_name.to_string(), span })?;
        return Ok(Some(constructor(value)));
    }

    // Try subtype/figment phrases with function call syntax
    if let Some((_, default_var)) = SUBTYPE_PHRASES.iter().find(|(pn, _)| *pn == phrase_name) {
        let variable_name =
            if args.len() == 1 && !args[0].is_empty() { args[0] } else { default_var };
        let subtype = bindings
            .get_subtype(variable_name)
            .ok_or_else(|| UnresolvedVariable { name: variable_name.to_string(), span })?;
        return Ok(Some(ResolvedToken::Subtype(subtype)));
    }
    if let Some((_, default_var)) = FIGMENT_PHRASES.iter().find(|(pn, _)| *pn == phrase_name) {
        let variable_name =
            if args.len() == 1 && !args[0].is_empty() { args[0] } else { default_var };
        let figment_type = bindings
            .get_figment(variable_name)
            .ok_or_else(|| UnresolvedVariable { name: variable_name.to_string(), span })?;
        return Ok(Some(ResolvedToken::Figment(figment_type)));
    }

    // Fall back: try the argument as a phrase name
    if args.len() == 1 {
        let arg = args[0];

        // Try integer phrase
        if let Some((_, default_var, constructor)) = PHRASES.iter().find(|(pn, _, _)| *pn == arg) {
            let value = bindings
                .get_integer(default_var)
                .ok_or_else(|| UnresolvedVariable { name: default_var.to_string(), span })?;
            return Ok(Some(constructor(value)));
        }

        // Try subtype phrase
        if let Some((_, default_var)) = SUBTYPE_PHRASES.iter().find(|(pn, _)| *pn == arg) {
            let subtype = bindings
                .get_subtype(default_var)
                .ok_or_else(|| UnresolvedVariable { name: default_var.to_string(), span })?;
            return Ok(Some(ResolvedToken::Subtype(subtype)));
        }

        // Try figment phrase
        if let Some((_, default_var)) = FIGMENT_PHRASES.iter().find(|(pn, _)| *pn == arg) {
            let figment_type = bindings
                .get_figment(default_var)
                .ok_or_else(|| UnresolvedVariable { name: default_var.to_string(), span })?;
            return Ok(Some(ResolvedToken::Figment(figment_type)));
        }

        // Try numbered variant for the argument
        if let Some((base, suffix)) = split_numbered_suffix(arg) {
            if base == "e" {
                let value = bindings
                    .get_integer(arg)
                    .ok_or_else(|| UnresolvedVariable { name: arg.to_string(), span })?;
                return match suffix {
                    "1" => Ok(Some(ResolvedToken::Mode1Energy(value))),
                    "2" => Ok(Some(ResolvedToken::Mode2Energy(value))),
                    _ => Ok(Some(ResolvedToken::Energy(value))),
                };
            }
            if let Some((_, _, constructor)) = PHRASES.iter().find(|(pn, _, _)| *pn == base) {
                let value = bindings
                    .get_integer(arg)
                    .ok_or_else(|| UnresolvedVariable { name: arg.to_string(), span })?;
                return Ok(Some(constructor(value)));
            }
        }
    }

    Ok(None)
}

fn split_numbered_suffix(name: &str) -> Option<(&str, &str)> {
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
