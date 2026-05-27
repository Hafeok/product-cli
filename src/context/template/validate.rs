//! Template validation — closed-allowlist checks producing E030 findings.

use super::loader::Template;

/// Recognised structural format values.
pub const ALLOWED_STRUCTURES: &[&str] = &["xml", "markdown", "yaml", "json", "plain"];

/// Closed-allowlist of section names the renderer recognises (FT-063).
/// FT-071 adds `patterns` for the pattern context section.
pub const ALLOWED_SECTIONS: &[&str] = &[
    "task",
    "feature",
    "deliverables",
    "governing_adrs",
    "test_criteria",
    "dependencies",
    "linked_documentation",
    "constraints",
    "bundle_metrics",
    "patterns",
];

pub const ALLOWED_ADRS_ORDERED_BY: &[&str] = &["centrality", "id"];
pub const ALLOWED_TCS_ORDERED_BY: &[&str] = &["type", "id"];

/// Validate a parsed template. Returns `Ok(())` when every closed-allowlist
/// check passes; otherwise returns a single human-readable description of
/// the failure (the caller wraps this into the E030 surface).
pub fn validate_template(t: &Template) -> Result<(), String> {
    if t.template.name.trim().is_empty() {
        return Err("template.name must not be empty".to_string());
    }
    if !ALLOWED_STRUCTURES.contains(&t.format.structure.as_str()) {
        return Err(format!(
            "format.structure {:?} is not one of {:?}",
            t.format.structure, ALLOWED_STRUCTURES,
        ));
    }
    if t.ordering.sections.is_empty() {
        return Err("ordering.sections must contain at least one section".to_string());
    }
    for s in &t.ordering.sections {
        if !ALLOWED_SECTIONS.contains(&s.as_str()) {
            return Err(format!(
                "ordering.sections contains unknown section {:?}; allowed: {:?}",
                s, ALLOWED_SECTIONS,
            ));
        }
    }
    if !ALLOWED_ADRS_ORDERED_BY.contains(&t.ordering.adrs_ordered_by.as_str()) {
        return Err(format!(
            "ordering.adrs_ordered_by {:?} is not one of {:?}",
            t.ordering.adrs_ordered_by, ALLOWED_ADRS_ORDERED_BY,
        ));
    }
    if !ALLOWED_TCS_ORDERED_BY.contains(&t.ordering.tcs_ordered_by.as_str()) {
        return Err(format!(
            "ordering.tcs_ordered_by {:?} is not one of {:?}",
            t.ordering.tcs_ordered_by, ALLOWED_TCS_ORDERED_BY,
        ));
    }
    Ok(())
}
