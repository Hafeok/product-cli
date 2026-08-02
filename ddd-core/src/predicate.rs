//! Predicate entries — definitional acceptance relations, never statused.
//!
//! Mirrors `docs/predicate-format.md` v1: one YAML file per predicate under
//! `.ddd/predicates/`, wrapped in a top-level `predicate:` key. Closure
//! findings live in claims; a `status` field here is rejected structurally
//! (`deny_unknown_fields`), which is ontology rule 1 of PRD §6.

use serde::{Deserialize, Serialize};

/// Top-level file shape: `predicate-format.md` files the entry under a
/// `predicate:` key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateFile {
    pub predicate: Predicate,
}

/// A named, reusable acceptance relation `P(c, G)` over one artifact class.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Predicate {
    /// Stable identifier `pred/<domain>/<name>`; renames create a new entry.
    pub id: String,
    /// Entry format version this predicate declares (v1 is current).
    pub format: u32,
    /// Short human name.
    pub name: String,
    /// The acceptance relation: accept `c` iff <condition over c and G>.
    pub statement: String,
    /// What `c` ranges over — disambiguates same-named predicates.
    pub artifact_class: ArtifactClass,
    /// What the check must see; each item carries provenance.
    pub ground: Vec<Ground>,
    /// The declared bound; without this the entry is invalid.
    pub tolerance: String,
    /// What a reject produces — evidence a violation exists.
    pub rejection_witness: String,
    /// Predicate ids this one presupposes (catalog graph edges, acyclic).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Claim ids only — closure lives in the claim graph, never here.
    #[serde(default)]
    pub closure_claims: Vec<String>,
    /// For gameable predicates: the value proxied plus how they can diverge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// The closed artifact-class vocabulary from `predicate-format.md` v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactClass {
    Code,
    Configuration,
    Execution,
    Interface,
    Data,
    Composition,
}

/// One required ground item with its provenance class.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ground {
    /// The required information.
    pub fact: String,
    /// Where the fact comes from; `observed`/`inferred` ground cannot be
    /// closed at build time by any arrangement.
    pub provenance: Provenance,
}

/// The closed provenance vocabulary from `predicate-format.md` v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provenance {
    Controlled,
    Observed,
    Inferred,
    Institutional,
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
predicate:
  id: pred/data/shape-conformance
  format: 1
  name: Data-shape conformance
  statement: Accept payload c iff c conforms exactly to the declared schema in G.
  artifact_class: data
  ground:
    - fact: the declared schema for the boundary
      provenance: controlled
    - fact: the payload as received at the boundary
      provenance: observed
  tolerance: exact structural match; no coercion
  rejection_witness: the first violating field path
  depends_on:
    - pred/data/well-formedness
  closure_claims:
    - DDD-cat-01
  proxy_gap: Conformance proxies validity.
"#;

    #[test]
    fn parses_the_format_example() {
        let f: PredicateFile = serde_yaml::from_str(EXAMPLE).expect("parse");
        assert_eq!(f.predicate.id, "pred/data/shape-conformance");
        assert_eq!(f.predicate.artifact_class, ArtifactClass::Data);
        assert_eq!(f.predicate.ground[1].provenance, Provenance::Observed);
        assert_eq!(f.predicate.depends_on.len(), 1);
    }

    #[test]
    fn a_status_field_is_rejected_structurally() {
        let with_status = EXAMPLE.replace("format: 1", "format: 1\n  status: closed");
        let err = serde_yaml::from_str::<PredicateFile>(&with_status)
            .expect_err("status must be rejected");
        assert!(err.to_string().contains("unknown field `status`"), "{err}");
    }
}
