use std::fs;
#[cfg(target_os = "android")]
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};
#[cfg(target_os = "android")]
use zip::ZipArchive;

use crate::tabula_error::TabulaError;
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
    resource: Arc<FluentResource>,
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
    Ok(FluentStrings { resource: Arc::new(resource) })
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
    ) -> Result<String, String> {
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
    ) -> Result<String, String> {
        args.set("context", context.key());
        let mut bundle = FluentBundle::default();
        bundle.set_use_isolating(false);
        if bundle.add_resource(self.resource.clone()).is_err() {
            return Err("ERR3: Add Resource Failed".to_string());
        }
        if let Some(additional) = additional_resource
            && bundle.add_resource(additional).is_err()
        {
            return Err("ERR3: Add Resource Failed".to_string());
        }
        let Some(msg) = bundle.get_message(id) else {
            return Err(format!("ERR4: Missing Message '{id}'"));
        };
        let Some(pattern) = msg.value() else {
            return Err(format!("ERR5: Missing Value for '{id}'"));
        };
        let mut errors = vec![];
        let out = bundle.format_pattern(pattern, Some(&args), &mut errors).into_owned();
        if errors.is_empty() { Ok(out) } else { Err(format_error_details(&errors)) }
    }

    /// Formats a string for display using Fluent placeable resolution.
    ///
    /// This method takes an arbitrary string containing Fluent references and
    /// resolves all placeables against the loaded resource bundle.
    pub fn format_display_string(
        &self,
        s: &str,
        context: StringContext,
        args: FluentArgs,
    ) -> Result<String, String> {
        if s.trim().is_empty() {
            return Ok(s.to_string());
        }
        let ftl = format!("tmp-for-display = {s}");
        let temp_res = FluentResource::try_new(ftl).map_err(|(_, errors)| {
            let message = errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ");
            format!("ERR7: Fluent Parser Error: {message}")
        })?;
        self.format_with_resource("tmp-for-display", context, args, Some(Arc::new(temp_res)))
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
