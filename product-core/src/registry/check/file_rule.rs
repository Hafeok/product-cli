//! The file rule: one assertion or one decision per data file.
//!
//! Generalised from the template's original filename exemption to the class
//! rule it stood in for. A rule with an exception decays; the empty
//! founding-decision slot went with the tolerance, so the slot is a documented
//! path an instance's ratifier creates rather than an empty file CI carves out.

use super::{Finding, FindingKind};

/// Check one data file. Returns a finding when the file does not carry exactly
/// one assertion or exactly one decision.
pub fn check_file(path: &str, text: &str, base: &str) -> Vec<Finding> {
    let query = format!(
        "SELECT ?s WHERE {{ ?s a ?c . VALUES ?c {{ <{base}Assertion> <{base}Decision> }} }}"
    );
    let rows = match crate::pf::sparql_rules::select(text, &query) {
        Ok(rows) => rows,
        Err(e) => {
            return vec![Finding {
                kind: FindingKind::FileRule,
                file: Some(path.to_string()),
                focus: "parse".to_string(),
                message: format!("the file could not be read as Turtle: {e}"),
            }]
        }
    };
    let subjects: Vec<String> = rows.iter().filter_map(|r| r.get("s").cloned()).collect();
    match subjects.len() {
        1 => Vec::new(),
        n => vec![Finding {
            kind: FindingKind::FileRule,
            file: Some(path.to_string()),
            focus: subjects.join(", "),
            message: format!(
                "carries {n} assertion/decision typings — exactly one assertion or exactly one decision is required"
            ),
        }],
    }
}
