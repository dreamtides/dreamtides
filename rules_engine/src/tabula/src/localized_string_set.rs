use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt;
use std::rc::Rc;

use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::tabula_table::HasId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct StringId(pub Uuid);

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum LanguageId {
    English,
}

#[derive(Serialize, Deserialize)]
pub struct LocalizedStringSet {
    pub id: StringId,
    pub name: String,
    pub description: String,
    pub english: String,
    #[serde(skip, default)]
    bundle_cache: RefCell<BTreeMap<LanguageId, Rc<FluentResource>>>,
}

impl LocalizedStringSet {
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
    pub fn format_pattern(&self, language: LanguageId, args: FluentArgs) -> String {
        {
            let mut cache = self.bundle_cache.borrow_mut();
            match cache.entry(language) {
                Entry::Occupied(_) => {}
                Entry::Vacant(v) => {
                    let ftl = match language {
                        LanguageId::English => format!("m = {}", self.english),
                    };
                    let res = match FluentResource::try_new(ftl) {
                        Ok(r) => Rc::new(r),
                        Err(_) => return "ERR1: Invalid Resource".to_string(),
                    };
                    v.insert(res);
                }
            }
        }
        let res = match self.bundle_cache.borrow().get(&language) {
            Some(r) => r.clone(),
            None => return "ERR2: Missing Resource".to_string(),
        };
        let mut bundle = FluentBundle::default();
        if bundle.add_resource(res.as_ref()).is_err() {
            return "ERR3: Add Resource Failed".to_string();
        }
        let msg = match bundle.get_message("m") {
            Some(m) => m,
            None => return "ERR4: Missing Message".to_string(),
        };
        let pattern = match msg.value() {
            Some(p) => p,
            None => return "ERR5: Missing Value".to_string(),
        };
        let mut errors = vec![];
        let out = bundle.format_pattern(pattern, Some(&args), &mut errors).into_owned();
        if errors.is_empty() { out } else { format_error_details(&errors) }
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

impl Clone for LocalizedStringSet {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            english: self.english.clone(),
            bundle_cache: RefCell::new(BTreeMap::new()),
        }
    }
}

impl fmt::Debug for LocalizedStringSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalizedStringSet")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("english", &self.english)
            .finish()
    }
}

impl HasId<StringId> for LocalizedStringSet {
    type Id = StringId;

    fn id(&self) -> StringId {
        self.id
    }
}
