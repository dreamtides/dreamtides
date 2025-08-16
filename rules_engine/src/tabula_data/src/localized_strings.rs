use std::collections::HashMap;
use std::sync::Arc;

use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::tabula::TabulaBuildContext;
use crate::tabula_table::{HasId, Table};

/// A string identifier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct StringId(pub Uuid);

/// A language identifier.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum LanguageId {
    EnglishUnitedStates,
}

/// A collection of localized strings.
#[derive(Debug, Clone)]
pub struct LocalizedStrings {
    resource: Arc<FluentResource>,
    id_to_key: HashMap<StringId, String>,
    build_error: Option<String>,
}

/// Serialized representation of LocalizedStringSet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedStringSetRaw {
    pub id: StringId,
    pub name: String,
    pub description: String,
    pub en_us: String,
}

impl LocalizedStringSetRaw {
    pub fn string(&self, language: LanguageId) -> String {
        match language {
            LanguageId::EnglishUnitedStates => self.en_us.clone(),
        }
    }
}

impl HasId<StringId> for LocalizedStringSetRaw {
    type Id = StringId;

    fn id(&self) -> StringId {
        self.id
    }
}

pub fn build(
    context: &TabulaBuildContext,
    table: &Table<StringId, LocalizedStringSetRaw>,
) -> LocalizedStrings {
    let ftl = table
        .0
        .iter()
        .map(|row| {
            format!(
                "{} = {}\n",
                row.name,
                normalize_literal(row.string(context.current_language).as_str())
            )
        })
        .collect::<Vec<_>>()
        .join("");
    match FluentResource::try_new(ftl) {
        Ok(res) => LocalizedStrings {
            resource: Arc::new(res),
            id_to_key: table.0.iter().map(|row| (row.id, row.name.clone())).collect(),
            build_error: None,
        },
        Err((res, errs)) => LocalizedStrings {
            resource: Arc::new(res),
            id_to_key: table.0.iter().map(|row| (row.id, row.name.clone())).collect(),
            build_error: Some(
                errs.iter()
                    .map(|e| format!("ERR7: Fluent Parser Error: {e}"))
                    .collect::<Vec<_>>()
                    .join(" | "),
            ),
        },
    }
}

impl LocalizedStrings {
    /// Formats the localized string using Fluent. In the
    /// event of an error, a descriptive error code is returned instead.
    ///
    /// Error codes:
    /// - ERR1: Invalid Resource
    /// - ERR2: Missing Resource
    /// - ERR3: Add Resource Failed
    /// - ERR4: Missing Message
    /// - ERR5: Missing Value
    /// - ERR6: Fluent Formatting Error: Overriding {kind} id={id}
    /// - ERR7: Fluent Parser Error: {parser_error}
    /// - ERR8: Fluent Resolver Error: {resolver_error}
    pub fn format_pattern(&self, id: StringId, args: &FluentArgs) -> String {
        if let Some(e) = &self.build_error {
            return e.clone();
        }
        let mut bundle = FluentBundle::default();
        bundle.set_use_isolating(false);
        if bundle.add_resource(self.resource.as_ref()).is_err() {
            return "ERR3: Add Resource Failed".to_string();
        }
        let key = match self.id_to_key.get(&id) {
            Some(k) => k,
            None => return "ERR4: Missing Message".to_string(),
        };
        let msg = match bundle.get_message(key.as_str()) {
            Some(m) => m,
            None => return "ERR4: Missing Message".to_string(),
        };
        let pattern = match msg.value() {
            Some(p) => p,
            None => return "ERR5: Missing Value".to_string(),
        };
        let mut errors = vec![];
        let out = bundle.format_pattern(pattern, Some(args), &mut errors).into_owned();
        if errors.is_empty() { out } else { format_error_details(&errors) }
    }
}

fn normalize_literal(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with("\\u[") && trimmed.ends_with("]") && trimmed.len() > 4 {
        let hex = &trimmed[3..trimmed.len() - 1];
        match u32::from_str_radix(hex, 16).ok().and_then(std::char::from_u32) {
            Some(ch) => ch.to_string(),
            None => s.to_string(),
        }
    } else {
        s.to_string()
    }
}

fn format_error_details(errors: &[FluentError]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for e in errors {
        match e {
            FluentError::Overriding { kind, id } => {
                parts.push(format!("ERR6: Fluent Formatting Error: Overriding {kind} id={id}"));
            }
            FluentError::ParserError(pe) => {
                parts.push(format!("ERR7: Fluent Parser Error: {pe}"));
            }
            FluentError::ResolverError(re) => {
                parts.push(format!("ERR8: Fluent Resolver Error: {re}"));
            }
        }
    }
    if parts.is_empty() { "ERR6: Fluent Formatting Error".to_string() } else { parts.join(" | ") }
}
