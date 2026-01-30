use std::fs;
#[cfg(target_os = "android")]
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use serde::de::DeserializeOwned;
#[cfg(target_os = "android")]
use zip::ZipArchive;

use crate::card_definition_raw::CardDefinitionRaw;
use crate::tabula_error::TabulaError;

/// Wrapper for deserializing card arrays from TOML files using `[[cards]]`
/// syntax.
#[derive(Debug, Deserialize)]
pub struct CardsFile {
    /// The array of raw card definitions.
    pub cards: Vec<CardDefinitionRaw>,
}

/// Wrapper for deserializing test card arrays from TOML files using
/// `[[test-cards]]` syntax.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestCardsFile {
    /// The array of raw test card definitions.
    pub test_cards: Vec<CardDefinitionRaw>,
}

/// Wrapper for deserializing dreamwell card arrays from TOML files using
/// `[[dreamwell]]` syntax.
#[derive(Debug, Deserialize)]
pub struct DreamwellFile {
    /// The array of raw dreamwell card definitions.
    pub dreamwell: Vec<CardDefinitionRaw>,
}

/// Wrapper for deserializing test dreamwell card arrays from TOML files using
/// `[[test-dreamwell]]` syntax.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestDreamwellFile {
    /// The array of raw test dreamwell card definitions.
    pub test_dreamwell: Vec<CardDefinitionRaw>,
}

/// Raw representation of a card effect row from TOML.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardEffectRowRaw {
    /// The card ID this effect applies to.
    pub card_id: String,
    /// The type of effect (e.g., "FireProjectile", "DissolveTargets").
    pub effect_type: String,
    /// The trigger condition for the effect.
    pub effect_trigger: String,
    /// Source of the projectile (for projectile effects).
    pub projectile_source: Option<String>,
    /// Target of the projectile (for projectile effects).
    pub projectile_target: Option<String>,
    /// Asset path for the projectile prefab.
    pub projectile_address: Option<String>,
    /// Sound to play when firing the projectile.
    pub projectile_fire_sound: Option<String>,
    /// Sound to play on projectile impact.
    pub projectile_impact_sound: Option<String>,
    /// Material for dissolve effects.
    pub dissolve_material: Option<String>,
    /// Color for dissolve effects.
    pub dissolve_color: Option<String>,
    /// Sound for dissolve effects.
    pub dissolve_sound: Option<String>,
    /// Target for display effects.
    pub effect_target: Option<String>,
    /// Asset path for display effect prefab.
    pub effect_address: Option<String>,
    /// Duration of the display effect in milliseconds.
    pub effect_duration_milliseconds: Option<i64>,
    /// Scale multiplier for display effects.
    pub effect_scale: Option<f64>,
    /// Sound for display effects.
    pub effect_sound: Option<String>,
    /// Target objects for card trail effects.
    pub card_trail_targets: Option<String>,
    /// Asset path for card trail prefab.
    pub card_trail_address: Option<String>,
    /// Duration of the card trail in milliseconds.
    pub trail_duration_milliseconds: Option<i64>,
}

/// Wrapper for deserializing card effect arrays from TOML files using
/// `[[card-fx]]` syntax.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardEffectsFile {
    /// The array of card effect rows.
    pub card_fx: Vec<CardEffectRowRaw>,
}

/// Raw representation of a card list row from TOML.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardListRowRaw {
    /// The name of the card list.
    pub list_name: String,
    /// The type of cards in the list (e.g., "DreamwellCardId").
    pub list_type: String,
    /// The card ID to include in the list.
    pub card_id: String,
    /// The number of copies of this card in the list.
    pub copies: i32,
}

/// Wrapper for deserializing card list arrays from TOML files using
/// `[[card-lists]]` syntax.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardListsFile {
    /// The array of card list rows.
    pub card_lists: Vec<CardListRowRaw>,
}

/// Loads and parses a TOML file from the filesystem into the specified type.
///
/// On Android, this function automatically handles loading from APK assets
/// when the path starts with `jar:file:`.
pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T, TabulaError> {
    let contents = read_file_contents(path)?;
    parse_toml(&contents, path)
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
    fs::read_to_string(path).map_err(|e| TabulaError::TomlParse {
        file: path.to_path_buf(),
        line: None,
        message: format!("Failed to read file: {e}"),
    })
}

/// Reads a file from an Android APK archive.
///
/// Handles jar:file: URLs in the format:
/// `jar:file:///path/to/base.apk!/assets/path/to/file.toml`
#[cfg(target_os = "android")]
fn read_android_asset(jar_url: &str, path: &Path) -> Result<String, TabulaError> {
    let without_prefix =
        jar_url.strip_prefix("jar:file:").ok_or_else(|| TabulaError::TomlParse {
            file: path.to_path_buf(),
            line: None,
            message: "Android jar URL missing jar:file: prefix".to_string(),
        })?;
    let bang_index = without_prefix.find("!/").ok_or_else(|| TabulaError::TomlParse {
        file: path.to_path_buf(),
        line: None,
        message: format!("Malformed Android jar URL: {jar_url}"),
    })?;
    let (apk_path, entry_path_with_slash) = without_prefix.split_at(bang_index);
    let entry_path = &entry_path_with_slash[2..];
    let mut file = std::fs::File::open(apk_path).map_err(|e| TabulaError::TomlParse {
        file: path.to_path_buf(),
        line: None,
        message: format!("Failed to open APK: {e}"),
    })?;
    let mut zip = ZipArchive::new(&mut file).map_err(|e| TabulaError::TomlParse {
        file: path.to_path_buf(),
        line: None,
        message: format!("Failed to read APK zip: {e}"),
    })?;
    let mut zip_file = zip.by_name(entry_path).map_err(|e| TabulaError::TomlParse {
        file: path.to_path_buf(),
        line: None,
        message: format!("File not found in APK: {e}"),
    })?;
    let mut buf = Vec::new();
    zip_file.read_to_end(&mut buf).map_err(|e| TabulaError::TomlParse {
        file: path.to_path_buf(),
        line: None,
        message: format!("Failed to read file from APK: {e}"),
    })?;
    String::from_utf8(buf).map_err(|e| TabulaError::TomlParse {
        file: path.to_path_buf(),
        line: None,
        message: format!("Invalid UTF-8 in file: {e}"),
    })
}

/// On non-Android platforms, jar:file: URLs are not supported.
#[cfg(not(target_os = "android"))]
fn read_android_asset(jar_url: &str, path: &Path) -> Result<String, TabulaError> {
    Err(TabulaError::TomlParse {
        file: path.to_path_buf(),
        line: None,
        message: format!("Android jar:file: URLs not supported on this platform: {jar_url}"),
    })
}

/// Parses a TOML string into the specified type.
fn parse_toml<T: DeserializeOwned>(contents: &str, path: &Path) -> Result<T, TabulaError> {
    toml::from_str(contents).map_err(|e| {
        let line = e.span().map(|s| contents[..s.start].chars().filter(|&c| c == '\n').count() + 1);
        TabulaError::TomlParse { file: path.to_path_buf(), line, message: e.message().to_string() }
    })
}
