use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::tabula_table::{HasId, Table};

/// A string identifier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct StringId(pub Uuid);

/// A localized string with arguments.
///
/// Can be rendered to an appropriate target language using
/// [`LocalizedStrings::format_localized_string`].
pub struct LocalizedString {
    pub id: StringId,
    pub args: FluentArgs<'static>,
}

/// A language identifier.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum LanguageId {
    English,
}

/// A collection of localized strings.
#[derive(Clone, Debug)]
pub struct LocalizedStrings {
    pub table: Table<StringId, LocalizedStringSet>,
    pub bundle_cache: RefCell<BTreeMap<LanguageId, Rc<FluentResource>>>,
}

/// A set of localizations for a given string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedStringSet {
    pub id: StringId,
    pub name: String,
    pub description: String,
    pub english: String,
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
        Ok(LocalizedStrings { table, bundle_cache: RefCell::new(BTreeMap::new()) })
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
        if !self.bundle_cache.borrow().contains_key(&language) {
            let mut ftl = String::new();
            for row in &self.table.0 {
                ftl.push_str(&format!(
                    "{} = {}\n",
                    row.name,
                    normalize_literal(row.english.as_str())
                ));
            }
            let res = match FluentResource::try_new(ftl) {
                Ok(r) => Rc::new(r),
                Err(_) => return "ERR1: Invalid Resource".to_string(),
            };
            self.bundle_cache.borrow_mut().insert(language, res);
        }
        let res = match self.bundle_cache.borrow().get(&language) {
            Some(r) => r.clone(),
            None => return "ERR2: Missing Resource".to_string(),
        };
        let mut bundle = FluentBundle::default();
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

    /// Formats the localized string for the given language using Fluent. In the
    /// event of an error, a descriptive error code is returned instead.
    ///
    /// See [`Self::format_pattern`] for more details.
    pub fn format_localized_string(
        &self,
        language: LanguageId,
        string: &LocalizedString,
    ) -> String {
        self.format_pattern(language, string.id, &string.args)
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
