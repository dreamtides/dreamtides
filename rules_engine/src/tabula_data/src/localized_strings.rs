use std::collections::HashMap;
use std::sync::Arc;

use core_data::initialization_error::{ErrorCode, InitializationError};
use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::tabula::TabulaBuildContext;
use crate::tabula_table::{HasId, Table};

/// A string identifier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct StringId(pub Uuid);

/// Describes the context in which a string is used.
///
/// Used to e.g. correctly determine colors colored elements.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum StringContext {
    Interface,
    CardText,
}

/// A language identifier.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum LanguageId {
    EnglishUnitedStates,
}

/// A collection of localized strings.
#[derive(Debug, Clone, Default)]
pub struct LocalizedStrings {
    resource: Option<Arc<FluentResource>>,
    id_to_key: HashMap<StringId, String>,
}

/// Serialized representation of LocalizedStringSet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedStringSetRaw {
    pub id: StringId,
    pub name: String,
    pub description: String,
    pub en_us: String,
}

pub fn build(
    context: &TabulaBuildContext,
    table: &Table<StringId, LocalizedStringSetRaw>,
) -> Result<LocalizedStrings, Vec<InitializationError>> {
    let mut errors: Vec<InitializationError> = Vec::new();

    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (row_index, row) in table.as_slice().iter().enumerate() {
        let c = seen.get(row.name.as_str()).copied().unwrap_or(0) + 1;
        if c > 1 {
            let mut e = InitializationError::with_name(
                ErrorCode::FluentFormattingError,
                format!("Duplicate message id {}", row.name),
            );
            e.tabula_sheet = Some(String::from("strings"));
            e.tabula_column = Some(String::from("name"));
            e.tabula_row = Some(row_index);
            errors.push(e);
        }
        seen.insert(row.name.as_str(), c);
    }

    let mut ftl_lines: Vec<String> = Vec::new();
    let mut valid_indices: Vec<usize> = Vec::new();
    for (row_index, row) in table.as_slice().iter().enumerate() {
        match validate_row_and_build_ftl_line(row_index, row, context.current_language) {
            Ok(line) => {
                ftl_lines.push(line);
                valid_indices.push(row_index);
            }
            Err(mut row_errs) => errors.append(&mut row_errs),
        }
    }
    let ftl = ftl_lines.join("");
    let (resource, parser_errs_opt) = match FluentResource::try_new(ftl) {
        Ok(res) => (res, None),
        Err((res, errs)) => (res, Some(errs)),
    };

    if let Some(parser_errs) = parser_errs_opt {
        for e in parser_errs {
            let mut ierr = InitializationError::with_details(
                ErrorCode::FluentParserError,
                String::from("Fluent Parser Error"),
                e.to_string(),
            );
            ierr.tabula_sheet = Some(String::from("strings"));
            errors.push(ierr);
        }
    }

    let mut bundle = FluentBundle::default();
    bundle.set_use_isolating(false);
    if let Err(bundle_errs) = bundle.add_resource(&resource) {
        for e in bundle_errs {
            let mut ierr = InitializationError::with_details(
                ErrorCode::FluentAddResourceError,
                String::from("Fluent Add Resource Error"),
                match e {
                    FluentError::Overriding { kind, id } => format!("overriding {kind} id={id}"),
                    FluentError::ParserError(pe) => pe.to_string(),
                    FluentError::ResolverError(re) => re.to_string(),
                },
            );
            ierr.tabula_sheet = Some(String::from("strings"));
            errors.push(ierr);
        }
    }

    for (row_index, row) in table.as_slice().iter().enumerate() {
        if !valid_indices.contains(&row_index) {
            continue;
        }
        if row.name.starts_with("-") {
            // Fluent terms cannot be directly retrieved
            continue;
        }

        match bundle.get_message(row.name.as_str()) {
            Some(m) => {
                if m.value().is_none() {
                    let mut ierr = InitializationError::with_name(
                        ErrorCode::FluentMissingValue,
                        row.name.clone(),
                    );
                    ierr.tabula_sheet = Some(String::from("strings"));
                    ierr.tabula_column = Some(String::from("en_us"));
                    ierr.tabula_row = Some(row_index);
                    errors.push(ierr);
                }
            }
            None => {
                let mut ierr = InitializationError::with_details(
                    ErrorCode::FluentMissingMessage,
                    row.name.clone(),
                    "Message not found in Fluent during validation",
                );
                ierr.tabula_sheet = Some(String::from("strings"));
                ierr.tabula_column = Some(String::from("name"));
                ierr.tabula_row = Some(row_index);
                errors.push(ierr);
            }
        }
    }

    let ls = LocalizedStrings {
        resource: Some(Arc::new(resource)),
        id_to_key: table
            .0
            .iter()
            .enumerate()
            .filter(|(i, _)| valid_indices.contains(i))
            .map(|(_, row)| (row.id, row.name.clone()))
            .collect(),
    };
    if errors.is_empty() { Ok(ls) } else { Err(errors) }
}

impl StringContext {
    pub fn key(&self) -> &'static str {
        match self {
            StringContext::Interface => "interface",
            StringContext::CardText => "card-text",
        }
    }
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

impl LocalizedStrings {
    fn resource(&self) -> Arc<FluentResource> {
        self.resource.clone().expect("Resource is not set")
    }

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
    pub fn format_pattern(
        &self,
        id: StringId,
        context: StringContext,
        mut args: FluentArgs,
    ) -> String {
        args.set("context", context.key());
        let mut bundle = FluentBundle::default();
        bundle.set_use_isolating(false);
        if bundle.add_resource(self.resource()).is_err() {
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
        let out = bundle.format_pattern(pattern, Some(&args), &mut errors).into_owned();
        if errors.is_empty() { out } else { format_error_details(&errors) }
    }

    /// Formats a string for display in the currently selected language with the
    /// given arguments, resolving all known Fluent placeables.
    pub fn format_display_string(
        &self,
        s: &str,
        context: StringContext,
        mut args: FluentArgs,
    ) -> String {
        if s.trim().is_empty() {
            return s.to_string();
        }

        args.set("context", context.key());
        let mut bundle = FluentBundle::default();
        bundle.set_use_isolating(false);
        if bundle.add_resource(self.resource()).is_err() {
            return "ERR3: Add Resource Failed".to_string();
        }
        let ftl = format!("tmp-for-display = {s}");
        let (temp_res, parser_errs_opt) = match FluentResource::try_new(ftl) {
            Ok(res) => (res, None),
            Err((res, errs)) => (res, Some(errs)),
        };
        if let Some(parser_errs) = parser_errs_opt {
            return format_error_details(
                &parser_errs.into_iter().map(FluentError::ParserError).collect::<Vec<_>>(),
            );
        }
        if bundle.add_resource(Arc::new(temp_res)).is_err() {
            return "ERR3: Add Resource Failed".to_string();
        }
        let msg = match bundle.get_message("tmp-for-display") {
            Some(m) => m,
            None => return "ERR4: Missing Message 'tmp-for-display'".to_string(),
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

fn validate_row_and_build_ftl_line(
    row_index: usize,
    row: &LocalizedStringSetRaw,
    language: LanguageId,
) -> Result<String, Vec<InitializationError>> {
    let tmp = format!("tmp = {}", row.string(language));
    match FluentResource::try_new(tmp.clone()) {
        Ok(_) => {}
        Err((_res, errs)) => {
            let mut out: Vec<InitializationError> = Vec::new();
            for e in errs {
                let mut ierr = InitializationError::with_details(
                    ErrorCode::FluentParserError,
                    String::from("Fluent Parser Error"),
                    format!("{e} in parsing {tmp:?}"),
                );
                ierr.tabula_sheet = Some(String::from("strings"));
                ierr.tabula_column = Some(String::from("en_us"));
                ierr.tabula_row = Some(row_index);
                out.push(ierr);
            }
            return Err(out);
        }
    }

    Ok(format!("{} = {}\n", row.name, row.string(language)))
}
