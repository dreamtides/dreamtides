use tracing::debug;

use crate::lint::rule_engine::{LintContext, LintDocument, LintResult, LintRule};

/// Reserved words that cannot appear in skill names.
const RESERVED_WORDS: &[&str] = &["anthropic", "claude"];

/// S001: Name contains reserved word.
///
/// Skill name cannot contain "anthropic" or "claude".
pub struct NameContainsReservedWordRule;

/// S002: Description empty.
///
/// Skill must have non-empty description.
pub struct DescriptionEmptyRule;

/// S003: Name contains XML.
///
/// Skill name cannot contain XML-like characters (< or >).
pub struct NameContainsXmlRule;

/// Returns all skill-specific lint rules (S001-S003).
pub fn all_skill_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NameContainsReservedWordRule),
        Box::new(DescriptionEmptyRule),
        Box::new(NameContainsXmlRule),
    ]
}

impl LintRule for NameContainsReservedWordRule {
    fn codes(&self) -> &[&str] {
        &["S001"]
    }

    fn name(&self) -> &str {
        "name-contains-reserved-word"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        if !document.frontmatter.skill {
            return vec![];
        }

        let name_lower = document.frontmatter.name.to_lowercase();
        for reserved in RESERVED_WORDS {
            if name_lower.contains(reserved) {
                let message = format!("skill name cannot contain '{reserved}'");
                debug!(path = %doc.row.path, name = %document.frontmatter.name, reserved, "Skill name contains reserved word");
                return vec![LintResult::error("S001", &doc.row.path, message)];
            }
        }

        vec![]
    }
}

impl LintRule for DescriptionEmptyRule {
    fn codes(&self) -> &[&str] {
        &["S002"]
    }

    fn name(&self) -> &str {
        "description-empty"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        if !document.frontmatter.skill {
            return vec![];
        }

        if document.frontmatter.description.trim().is_empty() {
            let message = "skill must have non-empty description";
            debug!(path = %doc.row.path, "Skill has empty description");
            return vec![LintResult::error("S002", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for NameContainsXmlRule {
    fn codes(&self) -> &[&str] {
        &["S003"]
    }

    fn name(&self) -> &str {
        "name-contains-xml"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        if !document.frontmatter.skill {
            return vec![];
        }

        let name = &document.frontmatter.name;
        if name.contains('<') || name.contains('>') {
            let message = "skill name cannot contain XML tags";
            debug!(path = %doc.row.path, name, "Skill name contains XML characters");
            return vec![LintResult::error("S003", &doc.row.path, message)];
        }

        vec![]
    }
}
