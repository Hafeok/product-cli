//! Authoring completeness join across every connected scope (§14.4).
//!
//! A What is complete when every required element kind is authored by *some*
//! sanctioned tool within its scope. The join reports, per required kind:
//! `covered` (by whom — partial authors compose), `coverable-but-unauthored`
//! (a connected tool's scope includes it, but no one authored it yet), or
//! `uncovered` (no connected tool's scope includes it — an author is missing).

use std::collections::{BTreeMap, HashSet};

use serde::Serialize;

use super::authoring_scope::AuthoringScope;

/// The coverage verdict for one required kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum Coverage {
    /// Authored, by these tools (partial authors compose).
    Covered { by: Vec<String> },
    /// A connected tool could author it, but none has yet.
    CoverableButUnauthored { candidates: Vec<String> },
    /// No connected tool's scope includes it — a sanctioned author is needed.
    Uncovered { detail: String },
}

impl Coverage {
    pub fn is_covered(&self) -> bool {
        matches!(self, Coverage::Covered { .. })
    }
    /// A short status label (`covered` | `coverable-but-unauthored` | `uncovered`).
    pub fn status(&self) -> &'static str {
        match self {
            Coverage::Covered { .. } => "covered",
            Coverage::CoverableButUnauthored { .. } => "coverable-but-unauthored",
            Coverage::Uncovered { .. } => "uncovered",
        }
    }
}

/// Run the completeness join. `authored_by_tool` maps a tool name to the kinds
/// it has actually authored (accepted). Returns `(complete, per-kind report)`;
/// `complete` is true when every required kind is `covered`.
pub fn completeness_join(
    required_kinds: &[String],
    scopes: &[AuthoringScope],
    authored_by_tool: &BTreeMap<String, HashSet<String>>,
) -> (bool, BTreeMap<String, Coverage>) {
    let mut report = BTreeMap::new();
    for kind in required_kinds {
        let authors: Vec<String> = scopes
            .iter()
            .filter(|s| s.authors.iter().any(|a| &a.kind == kind))
            .map(|s| s.tool.clone())
            .collect();
        let did: Vec<String> = authors
            .iter()
            .filter(|t| authored_by_tool.get(*t).is_some_and(|set| set.contains(kind)))
            .cloned()
            .collect();
        let coverage = if !did.is_empty() {
            Coverage::Covered { by: did }
        } else if !authors.is_empty() {
            Coverage::CoverableButUnauthored { candidates: authors }
        } else {
            Coverage::Uncovered {
                detail: "no connected tool's scope includes this kind — a sanctioned author is needed"
                    .to_string(),
            }
        };
        report.insert(kind.clone(), coverage);
    }
    let complete = report.values().all(Coverage::is_covered);
    (complete, report)
}
