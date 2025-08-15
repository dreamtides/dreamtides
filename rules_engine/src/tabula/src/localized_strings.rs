use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
#[derive(Debug)]
pub struct LocalizedStrings {
    pub table: Table<StringId, LocalizedStringSet>,
    pub bundle_cache: RwLock<BTreeMap<LanguageId, Arc<FluentResource>>>,
}

/// A set of localizations for a given string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedStringSet {
    pub id: StringId,
    pub name: String,
    pub description: String,
    pub en_us: String,
}

impl HasId<StringId> for LocalizedStringSet {
    type Id = StringId;

    fn id(&self) -> StringId {
        self.id
    }
}

impl Serialize for LocalizedStrings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.table.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LocalizedStrings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let table = Table::<StringId, LocalizedStringSet>::deserialize(deserializer)?;
        Ok(LocalizedStrings { table, bundle_cache: RwLock::new(BTreeMap::new()) })
    }
}

impl Clone for LocalizedStrings {
    fn clone(&self) -> Self {
        let table = self.table.clone();
        let cache = self.bundle_cache.read().unwrap().clone();
        LocalizedStrings { table, bundle_cache: RwLock::new(cache) }
    }
}

impl LocalizedStrings {
    /// Formats the localized string for the given language using Fluent. In the
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
    pub fn format_pattern(&self, language: LanguageId, id: StringId, args: &FluentArgs) -> String {
        if !self.bundle_cache.read().unwrap().contains_key(&language) {
            let mut ftl = String::new();
            for row in &self.table.0 {
                ftl.push_str(&format!(
                    "{} = {}\n",
                    row.name,
                    normalize_literal(row.en_us.as_str())
                ));
            }
            let res = match FluentResource::try_new(ftl) {
                Ok(r) => Arc::new(r),
                Err(_) => return "ERR1: Invalid Resource".to_string(),
            };
            self.bundle_cache.write().unwrap().insert(language, res);
        }
        let res = match self.bundle_cache.read().unwrap().get(&language) {
            Some(r) => r.clone(),
            None => return "ERR2: Missing Resource".to_string(),
        };
        let mut bundle = FluentBundle::default();
        bundle.set_use_isolating(false);
        if bundle.add_resource(res.as_ref()).is_err() {
            return "ERR3: Add Resource Failed".to_string();
        }
        let key = match self.table.get_optional(id) {
            Some(row) => row.name.clone(),
            None => return "ERR4: Missing Message".to_string(),
        };
        let msg = match bundle.get_message(&key) {
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
