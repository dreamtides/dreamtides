use std::fs;
#[cfg(target_os = "android")]
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};
use tabula_generated::string_id::StringId;
#[cfg(target_os = "android")]
use zip::ZipArchive;

use crate::tabula_error::TabulaError;

/// Mapping from RLF phrase names (with underscores) to Fluent message IDs
/// (with hyphens) for multi-word phrases.
const RLF_MULTI_WORD_PHRASES: &[(&str, &str)] = &[
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
    ("Reclaim_for_cost", "ReclaimForCost"),
    ("multiply_by", "MultiplyBy"),
];

/// Phrases where the Fluent message ID is the phrase name itself, not the
/// argument. These phrases have parameters in RLF that are consumed by the
/// Fluent message definition.
const PHRASE_NAME_IS_MESSAGE: &[&str] =
    &["foresee", "Foresee", "kindle", "Kindle", "MultiplyBy", "ReclaimForCost", "subtype"];

/// Mapping from RLF bare phrase names (without parentheses) to Fluent message
/// IDs for non-parameterized phrases where the naming convention differs.
const RLF_BARE_PHRASES: &[(&str, &str)] = &[
    ("energy_symbol", "energy-symbol"),
    ("choose_one", "ChooseOne"),
    ("judgment_phase_name", "JudgmentPhaseName"),
];

/// Describes the context in which a string is used.
///
/// Used to correctly determine formatting for colored elements and other
/// context-dependent rendering.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum StringContext {
    /// String is used in the game interface (menus, buttons, etc.).
    Interface,
    /// String is used in card text (rules text, abilities, etc.).
    CardText,
}

/// A collection of localized strings loaded from Fluent FTL files.
///
/// Wraps a `FluentBundle` and provides methods for formatting localized
/// strings with variable substitution.
#[derive(Debug)]
pub struct FluentStrings {
    /// The loaded Fluent resource.
    resource: Arc<FluentResource>,
    /// The path this resource was loaded from, for error reporting.
    source_path: PathBuf,
}

/// Loads Fluent strings from an FTL file at the specified path.
///
/// On Android, this function automatically handles loading from APK assets
/// when the path starts with `jar:file:`.
pub fn load(path: &Path) -> Result<FluentStrings, TabulaError> {
    let contents = read_file_contents(path)?;
    let resource = FluentResource::try_new(contents).map_err(|(_, errors)| {
        let message = errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ");
        TabulaError::FluentError {
            file: path.to_path_buf(),
            message_id: String::new(),
            message: format!("Failed to parse FTL file: {message}"),
        }
    })?;
    Ok(FluentStrings { resource: Arc::new(resource), source_path: path.to_path_buf() })
}

/// Reads file contents as a string, handling Android APK paths.
fn read_file_contents(path: &Path) -> Result<String, TabulaError> {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("jar:file:") {
        read_android_asset(&path_str, path)
    } else {
        read_filesystem_file(path)
    }
}

/// Reads a file from the standard filesystem.
fn read_filesystem_file(path: &Path) -> Result<String, TabulaError> {
    fs::read_to_string(path).map_err(|e| TabulaError::FluentError {
        file: path.to_path_buf(),
        message_id: String::new(),
        message: format!("Failed to read file: {e}"),
    })
}

/// Reads a file from an Android APK archive.
///
/// Handles jar:file: URLs in the format:
/// `jar:file:///path/to/base.apk!/assets/path/to/file.ftl`
#[cfg(target_os = "android")]
fn read_android_asset(jar_url: &str, path: &Path) -> Result<String, TabulaError> {
    let without_prefix =
        jar_url.strip_prefix("jar:file:").ok_or_else(|| TabulaError::FluentError {
            file: path.to_path_buf(),
            message_id: String::new(),
            message: "Android jar URL missing jar:file: prefix".to_string(),
        })?;
    let bang_index = without_prefix.find("!/").ok_or_else(|| TabulaError::FluentError {
        file: path.to_path_buf(),
        message_id: String::new(),
        message: format!("Malformed Android jar URL: {jar_url}"),
    })?;
    let (apk_path, entry_path_with_slash) = without_prefix.split_at(bang_index);
    let entry_path = &entry_path_with_slash[2..];
    let mut file = std::fs::File::open(apk_path).map_err(|e| TabulaError::FluentError {
        file: path.to_path_buf(),
        message_id: String::new(),
        message: format!("Failed to open APK: {e}"),
    })?;
    let mut zip = ZipArchive::new(&mut file).map_err(|e| TabulaError::FluentError {
        file: path.to_path_buf(),
        message_id: String::new(),
        message: format!("Failed to read APK zip: {e}"),
    })?;
    let mut zip_file = zip.by_name(entry_path).map_err(|e| TabulaError::FluentError {
        file: path.to_path_buf(),
        message_id: String::new(),
        message: format!("File not found in APK: {e}"),
    })?;
    let mut buf = Vec::new();
    zip_file.read_to_end(&mut buf).map_err(|e| TabulaError::FluentError {
        file: path.to_path_buf(),
        message_id: String::new(),
        message: format!("Failed to read file from APK: {e}"),
    })?;
    String::from_utf8(buf).map_err(|e| TabulaError::FluentError {
        file: path.to_path_buf(),
        message_id: String::new(),
        message: format!("Invalid UTF-8 in file: {e}"),
    })
}

/// On non-Android platforms, jar:file: URLs are not supported.
#[cfg(not(target_os = "android"))]
fn read_android_asset(jar_url: &str, path: &Path) -> Result<String, TabulaError> {
    Err(TabulaError::FluentError {
        file: path.to_path_buf(),
        message_id: String::new(),
        message: format!("Android jar:file: URLs not supported on this platform: {jar_url}"),
    })
}

impl StringContext {
    /// Returns the Fluent variable key for this context.
    pub fn key(&self) -> &'static str {
        match self {
            StringContext::Interface => "interface",
            StringContext::CardText => "card-text",
        }
    }
}

impl FluentStrings {
    /// Formats a localized string with the given message ID and arguments.
    ///
    /// Returns a formatted string or an error if the message ID is not found
    /// or formatting fails.
    pub fn format(
        &self,
        id: &str,
        context: StringContext,
        args: FluentArgs,
    ) -> Result<String, TabulaError> {
        self.format_with_resource(id, context, args, None)
    }

    /// Formats a localized string, using an optional additional resource.
    ///
    /// This is useful for formatting dynamic strings that contain Fluent
    /// references to the main resource bundle.
    fn format_with_resource(
        &self,
        id: &str,
        context: StringContext,
        mut args: FluentArgs,
        additional_resource: Option<Arc<FluentResource>>,
    ) -> Result<String, TabulaError> {
        args.set("context", context.key());
        let mut bundle = FluentBundle::default();
        bundle.set_use_isolating(false);
        if bundle.add_resource(self.resource.clone()).is_err() {
            return Err(TabulaError::FluentError {
                file: self.source_path.clone(),
                message_id: id.to_string(),
                message: "Failed to add resource to bundle".to_string(),
            });
        }
        if let Some(additional) = additional_resource
            && bundle.add_resource(additional).is_err()
        {
            return Err(TabulaError::FluentError {
                file: self.source_path.clone(),
                message_id: id.to_string(),
                message: "Failed to add additional resource to bundle".to_string(),
            });
        }
        let Some(msg) = bundle.get_message(id) else {
            return Err(TabulaError::FluentError {
                file: self.source_path.clone(),
                message_id: id.to_string(),
                message: format!("Message '{id}' not found"),
            });
        };
        let Some(pattern) = msg.value() else {
            return Err(TabulaError::FluentError {
                file: self.source_path.clone(),
                message_id: id.to_string(),
                message: format!("Message '{id}' has no value"),
            });
        };
        let mut errors = vec![];
        let out = bundle.format_pattern(pattern, Some(&args), &mut errors).into_owned();
        if errors.is_empty() {
            Ok(out)
        } else {
            Err(TabulaError::FluentError {
                file: self.source_path.clone(),
                message_id: id.to_string(),
                message: format_error_details(&errors),
            })
        }
    }

    /// Formats a string for display using Fluent placeable resolution.
    ///
    /// This method takes an arbitrary string containing Fluent references and
    /// resolves all placeables against the loaded resource bundle. Handles
    /// both legacy Fluent syntax and RLF function call syntax by converting
    /// RLF references to Fluent message references before formatting.
    pub fn format_display_string(
        &self,
        s: &str,
        context: StringContext,
        args: FluentArgs,
    ) -> Result<String, TabulaError> {
        if s.trim().is_empty() {
            return Ok(s.to_string());
        }
        let converted = convert_rlf_to_fluent(s);
        let ftl = format!("tmp-for-display = {converted}");
        let temp_res = FluentResource::try_new(ftl).map_err(|(_, errors)| {
            let message = errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ");
            TabulaError::FluentError {
                file: self.source_path.clone(),
                message_id: "tmp-for-display".to_string(),
                message: format!("Failed to parse display string: {message}"),
            }
        })?;
        self.format_with_resource("tmp-for-display", context, args, Some(Arc::new(temp_res)))
    }

    /// Formats a localized string using a [StringId].
    ///
    /// Returns the formatted string or an empty string if formatting fails.
    pub fn format_pattern(&self, id: StringId, context: StringContext, args: FluentArgs) -> String {
        self.format(id.identifier(), context, args).unwrap_or_default()
    }
}

/// Converts RLF function call syntax in serialized text to Fluent message
/// references so the string can be processed by the Fluent formatter.
fn convert_rlf_to_fluent(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut content = String::new();
            let mut depth = 1;
            for inner in chars.by_ref() {
                if inner == '{' {
                    depth += 1;
                } else if inner == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                content.push(inner);
            }
            if content.contains('(') {
                result.push('{');
                result.push_str(&convert_rlf_reference(&content));
                result.push('}');
            } else if let Some(fluent_name) = convert_rlf_bare_reference(&content) {
                result.push('{');
                result.push_str(&fluent_name);
                result.push('}');
            } else {
                result.push('{');
                result.push_str(&content);
                result.push('}');
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Converts a single RLF reference (the content between `{` and `}`) to the
/// equivalent Fluent message reference.
fn convert_rlf_reference(content: &str) -> String {
    let has_cap = content.starts_with("@cap ");
    let has_a = content.contains("@a ");
    let stripped = content.trim_start_matches("@cap ").trim_start_matches("@a ");

    let (core, selector) = if let Some(pos) = stripped.find(':') {
        (&stripped[..pos], Some(&stripped[pos + 1..]))
    } else {
        (stripped, None)
    };

    let Some(paren_start) = core.find('(') else {
        return content.to_string();
    };
    let Some(paren_end) = core.find(')') else {
        return content.to_string();
    };

    let phrase_name = core[..paren_start].trim();

    // Handle subtype special cases
    if phrase_name == "subtype" {
        if has_a && has_cap {
            return "ASubtype".to_string();
        }
        if has_a {
            return "a-subtype".to_string();
        }
        if selector == Some("other") {
            return "plural-subtype".to_string();
        }
        return "subtype".to_string();
    }

    // Check multi-word phrase table for underscore->hyphen conversion
    for (rlf_name, fluent_name) in RLF_MULTI_WORD_PHRASES {
        if phrase_name == *rlf_name {
            return fluent_name.to_string();
        }
    }

    // For phrases where the Fluent message ID is the phrase name itself
    if PHRASE_NAME_IS_MESSAGE.contains(&phrase_name) {
        return phrase_name.to_string();
    }

    // For capitalized multi-word phrases with underscores that aren't in the
    // table (shouldn't happen, but handle gracefully)
    if phrase_name.contains('_') {
        return phrase_name.replace('_', "-");
    }

    // Default: use the first argument as the Fluent message ID
    // (e.g., energy(e) -> e, cards(discards) -> discards, points(points) -> points)
    let args_str = &core[paren_start + 1..paren_end];
    let first_arg = args_str.split(',').next().unwrap_or(args_str).trim();
    if !first_arg.is_empty() {
        return first_arg.to_string();
    }

    content.to_string()
}

/// Converts a bare RLF phrase reference (without parentheses) to the
/// equivalent Fluent message ID, if a mapping exists.
fn convert_rlf_bare_reference(content: &str) -> Option<String> {
    RLF_BARE_PHRASES
        .iter()
        .find(|(rlf_name, _)| *rlf_name == content)
        .map(|(_, fluent_name)| fluent_name.to_string())
}

fn format_error_details(errors: &[FluentError]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for e in errors {
        match e {
            FluentError::Overriding { kind, id } => {
                parts.push(format!("Overriding {kind} with id '{id}'"));
            }
            FluentError::ParserError(pe) => {
                parts.push(format!("Parser error: {pe}"));
            }
            FluentError::ResolverError(re) => {
                parts.push(format!("Resolver error: {re}"));
            }
        }
    }
    if parts.is_empty() { "Unknown formatting error".to_string() } else { parts.join("; ") }
}

impl Default for FluentStrings {
    fn default() -> Self {
        let resource =
            FluentResource::try_new(String::new()).expect("Empty FTL string should always parse");
        Self { resource: Arc::new(resource), source_path: PathBuf::new() }
    }
}
