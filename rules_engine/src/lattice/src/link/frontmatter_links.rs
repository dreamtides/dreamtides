use serde_yaml::Value;

use crate::document::frontmatter_schema::Frontmatter;
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;

/// Type of link extracted from frontmatter fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterLinkType {
    /// Link from `blocking` field: this document blocks the target.
    Blocking,
    /// Link from `blocked-by` field: this document is blocked by the target.
    BlockedBy,
    /// Link from `discovered-from` field: provenance tracking.
    DiscoveredFrom,
    /// Link from a custom `*-id` or `*-ids` field.
    Custom,
}

/// A link extracted from a frontmatter field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterLink {
    /// The field name where this link was found (e.g., "blocking",
    /// "related-ids").
    pub source_field: String,
    /// The target document ID.
    pub target_id: LatticeId,
    /// The type of link.
    pub link_type: FrontmatterLinkType,
}

/// Result of extracting links from frontmatter.
#[derive(Debug, Clone)]
pub struct FrontmatterExtractionResult {
    /// All extracted links in field order.
    pub links: Vec<FrontmatterLink>,
}

/// Extracts all links from parsed frontmatter.
///
/// This extracts IDs from known link fields (`blocking`, `blocked-by`,
/// `discovered-from`) from the parsed Frontmatter struct.
pub fn extract(frontmatter: &Frontmatter) -> FrontmatterExtractionResult {
    let mut links = Vec::new();

    for id in &frontmatter.blocking {
        links.push(FrontmatterLink {
            source_field: "blocking".to_string(),
            target_id: id.clone(),
            link_type: FrontmatterLinkType::Blocking,
        });
    }

    for id in &frontmatter.blocked_by {
        links.push(FrontmatterLink {
            source_field: "blocked-by".to_string(),
            target_id: id.clone(),
            link_type: FrontmatterLinkType::BlockedBy,
        });
    }

    for id in &frontmatter.discovered_from {
        links.push(FrontmatterLink {
            source_field: "discovered-from".to_string(),
            target_id: id.clone(),
            link_type: FrontmatterLinkType::DiscoveredFrom,
        });
    }

    FrontmatterExtractionResult { links }
}

/// Extracts links from both parsed frontmatter and raw YAML for custom fields.
///
/// This extracts IDs from:
/// - Known fields: `blocking`, `blocked-by`, `discovered-from` (from
///   Frontmatter)
/// - Custom fields: Any field ending in `-id` or `-ids` (from raw YAML)
pub fn extract_with_custom_fields(
    frontmatter: &Frontmatter,
    raw_yaml: &str,
) -> Result<FrontmatterExtractionResult, LatticeError> {
    let mut result = extract(frontmatter);
    let custom_links = extract_custom_fields(raw_yaml)?;
    result.links.extend(custom_links);
    Ok(result)
}

/// Extracts links from custom `*-id` and `*-ids` fields in raw YAML.
fn extract_custom_fields(raw_yaml: &str) -> Result<Vec<FrontmatterLink>, LatticeError> {
    let mut links = Vec::new();

    let value: Value =
        serde_yaml::from_str(raw_yaml).map_err(|e| LatticeError::YamlParseError {
            path: std::path::PathBuf::from("<raw>"),
            reason: e.to_string(),
        })?;

    let Some(mapping) = value.as_mapping() else {
        return Ok(links);
    };

    for (key, val) in mapping {
        let Some(field_name) = key.as_str() else {
            continue;
        };

        if !is_custom_id_field(field_name) {
            continue;
        }

        extract_ids_from_value(field_name, val, &mut links)?;
    }

    Ok(links)
}

/// Returns true if the field name indicates a custom ID field.
fn is_custom_id_field(field_name: &str) -> bool {
    let known_fields = ["lattice-id", "parent-id", "blocking", "blocked-by", "discovered-from"];
    if known_fields.contains(&field_name) {
        return false;
    }
    field_name.ends_with("-id") || field_name.ends_with("-ids")
}

/// Extracts IDs from a YAML value (single string or array of strings).
fn extract_ids_from_value(
    field_name: &str,
    value: &Value,
    links: &mut Vec<FrontmatterLink>,
) -> Result<(), LatticeError> {
    match value {
        Value::String(s) => {
            let id = LatticeId::parse(s)?;
            links.push(FrontmatterLink {
                source_field: field_name.to_string(),
                target_id: id,
                link_type: FrontmatterLinkType::Custom,
            });
        }
        Value::Sequence(seq) => {
            for item in seq {
                if let Some(s) = item.as_str() {
                    let id = LatticeId::parse(s)?;
                    links.push(FrontmatterLink {
                        source_field: field_name.to_string(),
                        target_id: id,
                        link_type: FrontmatterLinkType::Custom,
                    });
                }
            }
        }
        _ => {}
    }
    Ok(())
}
