use std::collections::HashSet;
use std::path::Path;

use serde_yaml::{Error as YamlError, Value};

use crate::document::frontmatter_schema::Frontmatter;
use crate::error::error_types::LatticeError;

const FRONTMATTER_DELIMITER: &str = "---";
/// All valid keys that can appear in Lattice frontmatter.
const ALLOWED_KEYS: &[&str] = &[
    "lattice-id",
    "name",
    "description",
    "parent-id",
    "task-type",
    "priority",
    "labels",
    "blocking",
    "blocked-by",
    "discovered-from",
    "created-at",
    "updated-at",
    "closed-at",
    "skill",
];

/// Result of parsing a Lattice document's frontmatter.
#[derive(Debug, Clone)]
pub struct ParsedFrontmatter {
    /// The parsed and validated frontmatter data.
    pub frontmatter: Frontmatter,
    /// The original YAML string for round-trip preservation.
    pub raw_yaml: String,
    /// The markdown body content after the frontmatter.
    pub body: String,
}

/// Unknown keys found during frontmatter parsing.
#[derive(Debug, Clone)]
pub struct UnknownKey {
    /// The unknown key name.
    pub key: String,
    /// Suggestion for a similar valid key, if available.
    pub suggestion: Option<String>,
}

/// Parses frontmatter from markdown content.
pub fn parse(content: &str, path: &Path) -> Result<ParsedFrontmatter, LatticeError> {
    let (raw_yaml, body) = extract_yaml(content, path)?;
    let frontmatter = parse_yaml(&raw_yaml, path)?;
    Ok(ParsedFrontmatter { frontmatter, raw_yaml, body })
}

/// Parses frontmatter with detection of unknown keys.
pub fn parse_with_unknown_key_detection(
    content: &str,
    path: &Path,
) -> Result<(ParsedFrontmatter, Vec<UnknownKey>), LatticeError> {
    let (raw_yaml, body) = extract_yaml(content, path)?;
    let unknown_keys = detect_unknown_keys(&raw_yaml, path)?;
    let frontmatter = parse_yaml(&raw_yaml, path)?;
    Ok((ParsedFrontmatter { frontmatter, raw_yaml, body }, unknown_keys))
}

/// Serializes frontmatter to YAML string.
pub fn serialize(frontmatter: &Frontmatter) -> Result<String, LatticeError> {
    serde_yaml::to_string(frontmatter).map_err(|e| LatticeError::InvalidFrontmatter {
        id: frontmatter.lattice_id.to_string(),
        path: std::path::PathBuf::new(),
        reason: format!("failed to serialize frontmatter: {e}"),
    })
}

/// Formats a complete document with frontmatter and body.
pub fn format_document(frontmatter: &Frontmatter, body: &str) -> Result<String, LatticeError> {
    let yaml = serialize(frontmatter)?;
    let yaml_trimmed = yaml.trim();
    if body.is_empty() {
        Ok(format!("{FRONTMATTER_DELIMITER}\n{yaml_trimmed}\n{FRONTMATTER_DELIMITER}\n"))
    } else {
        Ok(format!("{FRONTMATTER_DELIMITER}\n{yaml_trimmed}\n{FRONTMATTER_DELIMITER}\n\n{body}"))
    }
}

/// Extracts the YAML frontmatter and body from markdown content.
fn extract_yaml(content: &str, path: &Path) -> Result<(String, String), LatticeError> {
    let content = content.trim_start_matches('\u{feff}');

    if !content.starts_with(FRONTMATTER_DELIMITER) {
        return Err(LatticeError::InvalidFrontmatter {
            id: String::new(),
            path: path.to_path_buf(),
            reason: "document must start with '---' frontmatter delimiter".to_string(),
        });
    }

    let after_opening = &content[FRONTMATTER_DELIMITER.len()..];
    let closing_pos = find_closing_delimiter(after_opening);

    let Some(closing_pos) = closing_pos else {
        return Err(LatticeError::InvalidFrontmatter {
            id: String::new(),
            path: path.to_path_buf(),
            reason: "missing closing '---' frontmatter delimiter".to_string(),
        });
    };

    let yaml_content = &after_opening[..closing_pos];
    let yaml_trimmed = yaml_content.trim();

    if yaml_trimmed.is_empty() {
        return Err(LatticeError::InvalidFrontmatter {
            id: String::new(),
            path: path.to_path_buf(),
            reason: "frontmatter cannot be empty".to_string(),
        });
    }

    let body_start = FRONTMATTER_DELIMITER.len() + closing_pos + FRONTMATTER_DELIMITER.len();
    let body = if body_start < content.len() {
        content[body_start..].trim_start_matches(['\n', '\r']).to_string()
    } else {
        String::new()
    };

    Ok((yaml_trimmed.to_string(), body))
}

/// Finds the position of the closing `---` delimiter.
fn find_closing_delimiter(content: &str) -> Option<usize> {
    let mut pos = 0;
    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed == FRONTMATTER_DELIMITER {
            return Some(pos);
        }
        pos += line.len();
    }

    let remaining = &content[pos..];
    if remaining.trim() == FRONTMATTER_DELIMITER {
        return Some(pos);
    }

    None
}

/// Parses a YAML string into Frontmatter.
fn parse_yaml(yaml: &str, path: &Path) -> Result<Frontmatter, LatticeError> {
    serde_yaml::from_str(yaml).map_err(|e| LatticeError::YamlParseError {
        path: path.to_path_buf(),
        reason: format_yaml_error(e),
    })
}

/// Formats a serde_yaml error into a user-friendly message.
fn format_yaml_error(error: YamlError) -> String {
    let location = error.location();
    match location {
        Some(loc) => format!("line {}, column {}: {}", loc.line(), loc.column(), error),
        None => error.to_string(),
    }
}

/// Detects unknown keys in YAML frontmatter.
fn detect_unknown_keys(yaml: &str, path: &Path) -> Result<Vec<UnknownKey>, LatticeError> {
    let value: Value = serde_yaml::from_str(yaml).map_err(|e| LatticeError::YamlParseError {
        path: path.to_path_buf(),
        reason: format_yaml_error(e),
    })?;

    let Some(mapping) = value.as_mapping() else {
        return Err(LatticeError::InvalidFrontmatter {
            id: String::new(),
            path: path.to_path_buf(),
            reason: "frontmatter must be a YAML mapping".to_string(),
        });
    };

    let allowed: HashSet<&str> = ALLOWED_KEYS.iter().copied().collect();
    let mut unknown = Vec::new();

    for key in mapping.keys() {
        let Some(key_str) = key.as_str() else {
            continue;
        };

        if !allowed.contains(key_str) {
            unknown.push(UnknownKey { key: key_str.to_string(), suggestion: suggest_key(key_str) });
        }
    }

    Ok(unknown)
}

/// Suggests a similar valid key for a typo.
fn suggest_key(unknown: &str) -> Option<String> {
    let unknown_lower = unknown.to_lowercase();

    for &allowed in ALLOWED_KEYS {
        if edit_distance(&unknown_lower, allowed) <= 2 {
            return Some(allowed.to_string());
        }
    }

    None
}

/// Computes the Levenshtein edit distance between two strings.
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}
