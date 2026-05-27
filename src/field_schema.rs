//! Front-matter field allowlists per artifact type (FT-062).
//!
//! Single source of truth for the recognised front-matter field names for
//! each artifact type. Both the request-validator (E026) and the
//! `product_schema` MCP tool consult this module so the contract is
//! introspectable and the validator and the schema tool can never diverge.
//!
//! The pseudo-field `body` is intentionally not in any list because it is a
//! virtual mutation target documented in ADR-038 decision 9 and handled as a
//! special case at the call site.

use crate::request::types::ArtifactType;

/// Recognised front-matter field names on a feature artifact.
///
/// These mirror the `#[serde(rename = ...)]` names on
/// `crate::types::FeatureFrontMatter`.
pub const FEATURE_FIELDS: &[&str] = &[
    "id",
    "title",
    "phase",
    "status",
    "depends-on",
    "adrs",
    "tests",
    "domains",
    "domains-acknowledged",
    "due-date",
    "bundle",
];

/// Recognised front-matter field names on an ADR artifact.
pub const ADR_FIELDS: &[&str] = &[
    "id",
    "title",
    "status",
    "features",
    "supersedes",
    "superseded-by",
    "domains",
    "scope",
    "content-hash",
    "amendments",
    "source-files",
    "removes",
    "deprecates",
];

/// Recognised front-matter field names on a TC artifact.
pub const TC_FIELDS: &[&str] = &[
    "id",
    "title",
    "type",
    "status",
    "validates",
    "phase",
    "content-hash",
    "runner",
    "runner-args",
    "runner-timeout",
    "requires",
    "observes",
    "last-run",
    "failure-message",
    "last-run-duration",
];

/// Recognised front-matter field names on a dependency artifact.
pub const DEP_FIELDS: &[&str] = &[
    "id",
    "title",
    "type",
    "source",
    "version",
    "status",
    "features",
    "adrs",
    "availability-check",
    "breaking-change-risk",
    "interface",
    "supersedes",
];

/// Recognised front-matter field names on a pattern artifact (FT-070).
pub const PATTERN_FIELDS: &[&str] = &[
    "id",
    "title",
    "status",
    "domains",
    "adrs",
    "requires",
    "examples",
    "deprecated-by",
];

/// Resolve the artifact type label string ("feature", "adr", "tc", "dep") to
/// its known-fields slice. Any other label returns `&[]`.
pub fn known_fields_for_label(label: &str) -> &'static [&'static str] {
    match label {
        "feature" => FEATURE_FIELDS,
        "adr" => ADR_FIELDS,
        "tc" => TC_FIELDS,
        "dep" => DEP_FIELDS,
        "pattern" => PATTERN_FIELDS,
        _ => &[],
    }
}

/// Resolve an `ArtifactType` to its known-fields slice.
pub fn known_fields_for(t: ArtifactType) -> &'static [&'static str] {
    match t {
        ArtifactType::Feature => FEATURE_FIELDS,
        ArtifactType::Adr => ADR_FIELDS,
        ArtifactType::Tc => TC_FIELDS,
        ArtifactType::Dep => DEP_FIELDS,
        ArtifactType::Pattern => PATTERN_FIELDS,
    }
}

/// True iff `field` is a recognised front-matter field for `artifact_type`,
/// or the pseudo-field `body` (ADR-038 decision 9).
///
/// Dot-notation paths are matched on the **first segment** — the leaf is
/// intentionally not checked because nested keys (domain names, dependency
/// interface fields, acknowledgement keys) are open vocabularies.
pub fn is_known_field(artifact_type: ArtifactType, field: &str) -> bool {
    let head = field.split('.').next().unwrap_or(field);
    if head == "body" {
        return true;
    }
    known_fields_for(artifact_type).contains(&head)
}

/// Suggest the closest known field name (Levenshtein distance ≤ 2).
/// Returns `None` if no candidate is within the threshold or if the input
/// itself matches an existing field.
pub fn suggest_closest(artifact_type: ArtifactType, field: &str) -> Option<&'static str> {
    let head = field.split('.').next().unwrap_or(field);
    if head == "body" || known_fields_for(artifact_type).contains(&head) {
        return None;
    }
    let mut best: Option<(&'static str, usize)> = None;
    for candidate in known_fields_for(artifact_type) {
        let d = levenshtein(head, candidate);
        match best {
            Some((_, current)) if d >= current => {}
            _ => best = Some((candidate, d)),
        }
    }
    best.and_then(|(c, d)| if d <= 2 { Some(c) } else { None })
}

/// Standard iterative Levenshtein distance.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let n = a.len();
    let m = b.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_has_depends_on_field() {
        assert!(FEATURE_FIELDS.contains(&"depends-on"));
    }

    #[test]
    fn adr_has_status_field() {
        assert!(ADR_FIELDS.contains(&"status"));
    }

    #[test]
    fn body_is_always_known() {
        assert!(is_known_field(ArtifactType::Feature, "body"));
        assert!(is_known_field(ArtifactType::Adr, "body"));
        assert!(is_known_field(ArtifactType::Tc, "body"));
        assert!(is_known_field(ArtifactType::Dep, "body"));
    }

    #[test]
    fn dot_notation_validates_head_only() {
        assert!(is_known_field(
            ArtifactType::Feature,
            "domains-acknowledged.security"
        ));
        assert!(is_known_field(
            ArtifactType::Dep,
            "interface.port"
        ));
    }

    #[test]
    fn unknown_field_rejected() {
        assert!(!is_known_field(ArtifactType::Feature, "dependsOn"));
        assert!(!is_known_field(ArtifactType::Feature, "totally-bogus"));
    }

    #[test]
    fn suggest_returns_close_match_for_camelcase_typo() {
        assert_eq!(
            suggest_closest(ArtifactType::Feature, "dependsOn"),
            Some("depends-on")
        );
    }

    #[test]
    fn suggest_returns_none_for_unknown_garbage() {
        assert_eq!(
            suggest_closest(ArtifactType::Feature, "completely-unrelated-name"),
            None
        );
    }

    #[test]
    fn suggest_returns_none_for_known_field() {
        assert_eq!(suggest_closest(ArtifactType::Feature, "depends-on"), None);
    }

    #[test]
    fn levenshtein_basic_cases() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("dependsOn", "depends-on"), 2);
    }
}
