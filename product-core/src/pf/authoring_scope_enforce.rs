//! Authoring-scope enforcement — accept in-scope authorship, reject the rest (§14.3).
//!
//! Everything a tool submits is checked against its declared scope. Authorship
//! of an excluded kind (or a kind not in `authors`) is rejected regardless of
//! content quality. What the adapter suspects needs authorship then splits along
//! the scope boundary into `unauthored-within-scope` (author it in the tool) vs
//! `outside-scope` (route to a sanctioned author).

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::authoring_scope::AuthoringScope;

/// One item in a tool submission: a What-element of some kind.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SubmissionItem {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
}

/// A tool's submission: what the adapter extracted as authored meaning, plus
/// what it suspects needs authorship (the pre-split annotation-needed list).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Submission {
    #[serde(default)]
    pub authored: Vec<SubmissionItem>,
    #[serde(rename = "unauthored-candidates", default)]
    pub unauthored_candidates: Vec<SubmissionItem>,
}

impl Submission {
    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| format!("invalid submission JSON: {e}"))
    }
}

/// A rejected item, carrying the scope-violation basis.
#[derive(Debug, Clone, Serialize)]
pub struct Rejected {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    pub basis: String,
}

/// A gap item, carrying the routed action.
#[derive(Debug, Clone, Serialize)]
pub struct Gap {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    pub action: String,
}

/// The enforcement findings (§14.3), one bucket per outcome.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EnforceFindings {
    pub accepted: Vec<SubmissionItem>,
    #[serde(rename = "rejected-out-of-scope")]
    pub rejected_out_of_scope: Vec<Rejected>,
    #[serde(rename = "unauthored-within-scope")]
    pub unauthored_within_scope: Vec<Gap>,
    #[serde(rename = "outside-scope")]
    pub outside_scope: Vec<Gap>,
}

/// Run the enforcement oracle. `valid` is true when nothing was rejected
/// out-of-scope (the What's own validity checks are separate, §14.3).
pub fn enforce(scope: &AuthoringScope, submission: &Submission) -> (bool, EnforceFindings) {
    let ok_kinds: HashSet<&str> = scope.authors.iter().map(|a| a.kind.as_str()).collect();
    let excluded: HashSet<&str> = scope.excluded.iter().map(|s| s.as_str()).collect();
    let mut f = EnforceFindings::default();

    for item in &submission.authored {
        let k = item.kind.as_str();
        if excluded.contains(k) {
            f.rejected_out_of_scope.push(Rejected {
                kind: item.kind.clone(),
                element: item.element.clone(),
                basis: format!(
                    "'{}' is excluded from authoring '{k}' — rejected regardless of content",
                    scope.tool
                ),
            });
        } else if ok_kinds.contains(k) {
            f.accepted.push(item.clone());
        } else {
            f.rejected_out_of_scope.push(Rejected {
                kind: item.kind.clone(),
                element: item.element.clone(),
                basis: format!("'{k}' is not in '{}''s declared authors list", scope.tool),
            });
        }
    }

    for item in &submission.unauthored_candidates {
        let k = item.kind.as_str();
        if ok_kinds.contains(k) {
            f.unauthored_within_scope.push(Gap {
                kind: item.kind.clone(),
                element: item.element.clone(),
                action: format!("authorable in {} — author it there (e.g. annotate)", scope.tool),
            });
        } else {
            f.outside_scope.push(Gap {
                kind: item.kind.clone(),
                element: item.element.clone(),
                action: "not authorable in this tool — route to a sanctioned author".to_string(),
            });
        }
    }

    let valid = f.rejected_out_of_scope.is_empty();
    (valid, f)
}
