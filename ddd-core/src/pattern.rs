//! Pattern-instance entries — declared instantiations of catalog predicates.
//!
//! One flat YAML file per instance under `.ddd/patterns/`. Detection never
//! files these (curation, not mining); an instance exists because someone
//! declared it, referencing the pattern predicate it instantiates (ontology
//! rule 5) with the pattern's obligations answered.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A declared pattern instance plus its obligation completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatternInstance {
    /// Entry format version (v1 is current).
    pub format: u32,
    /// Stable id `pat/<pattern>/<instance>`.
    pub id: String,
    /// The catalog predicate this instance instantiates (rule 5).
    pub pattern_predicate: String,
    /// Where the instance lives (e.g. a composition-root registration site).
    pub instance: String,
    /// The pattern's obligation list with this instance's answers
    /// (e.g. decorator: ordering, identity, forwarding completeness).
    #[serde(default)]
    pub obligations: BTreeMap<String, String>,
    /// The decision that introduced the pattern, when one is filed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_obligation_answers() {
        let p: PatternInstance = serde_yaml::from_str(
            "format: 1\nid: pat/decorator/logging\npattern_predicate: pred/composition/decorator\ninstance: Scrutor .Decorate in Program.cs\nobligations:\n  ordering: outermost\n  identity: preserved\n",
        )
        .expect("parse");
        assert_eq!(p.obligations.len(), 2);
        assert_eq!(p.pattern_predicate, "pred/composition/decorator");
    }
}
