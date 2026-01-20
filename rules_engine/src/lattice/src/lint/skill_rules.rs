use crate::lint::rule_engine::{LintContext, LintDocument, LintResult, LintRule};
use crate::skill::skill_validation;

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

        if let Some(error) = skill_validation::check_reserved_words(&document.frontmatter.name) {
            return vec![LintResult::error(error.code, &doc.row.path, error.message)];
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

        if let Some(error) =
            skill_validation::check_description_empty(&document.frontmatter.description)
        {
            return vec![LintResult::error(error.code, &doc.row.path, error.message)];
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

        if let Some(error) = skill_validation::check_xml_characters(&document.frontmatter.name) {
            return vec![LintResult::error(error.code, &doc.row.path, error.message)];
        }

        vec![]
    }
}
