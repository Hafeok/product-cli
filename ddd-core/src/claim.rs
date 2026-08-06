//! Closure-claim entries — truth-apt findings, the only status carriers.
//!
//! Mirrors the claim format specification v1: one flat YAML file per claim
//! under `.ddd/claims/` (no wrapper key), validated against the format
//! version the claim itself declares. The tool must host framework-repo
//! claims unconverted, so every spec field is present under its spec name;
//! `refines` is a tool addition for PRD §6 rule 2.

use serde::{Deserialize, Serialize};

/// One claim per file — compound propositions are split into several claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    /// Spec version this claim conforms to; mandatory.
    pub format: u32,
    /// Stable id `DDD-<area>-<n>`; never reused, retired claims keep theirs.
    pub id: String,
    /// The proposition.
    pub statement: String,
    /// The claim's epistemic status — claims alone carry one.
    pub status: ClaimStatus,
    /// What the status rests on; required for `reported` status or above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// The specific observation that would kill it; required while live.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub falsifier: Option<String>,
    /// Which paper/projection pays for this claim; `none` is legal.
    pub owner: String,
    /// Date of last content change — the claim's own version axis.
    pub changed: String,
    /// Optional revalidation cadence (format 2): the date by which the
    /// status must be rechecked; `report escapes` flags claims past it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revalidate_by: Option<String>,
    /// Claim ids this one presupposes (acyclic).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Claim ids this one refines (tool addition; targets must exist).
    #[serde(default)]
    pub refines: Vec<String>,
    /// The predicate id this claim is a closure finding about (tool addition).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicate: Option<String>,
}

/// Status ladder: projected < reported < established; retired is terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaimStatus {
    /// Clean derivation, unexercised.
    Projected,
    /// Exercised evidence.
    Reported,
    /// Checker-engaged.
    Established,
    /// Killed; the file is kept with the evidence that killed it.
    Retired,
}

impl ClaimStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Projected => "projected",
            Self::Reported => "reported",
            Self::Established => "established",
            Self::Retired => "retired",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC_SHAPED: &str = r#"
format: 1
id: DDD-cat-01
statement: TypeScript strict mode operationally closes shape conformance at compile time.
status: reported
evidence: exercised on two engagement repos
falsifier: a conforming payload rejected at a non-serialisation boundary
owner: none
changed: 2026-07-30
depends_on: []
"#;

    #[test]
    fn hosts_a_spec_shaped_claim_unconverted() {
        let c: Claim = serde_yaml::from_str(SPEC_SHAPED).expect("parse");
        assert_eq!(c.id, "DDD-cat-01");
        assert_eq!(c.status, ClaimStatus::Reported);
        assert_eq!(c.owner, "none");
        assert!(c.refines.is_empty());
    }

    #[test]
    fn a_format_two_claim_may_declare_a_revalidation_cadence() {
        let text = SPEC_SHAPED.replace("format: 1", "format: 2\nrevalidate_by: 2027-01-01");
        let c: Claim = serde_yaml::from_str(&text).expect("parse");
        assert_eq!(c.revalidate_by.as_deref(), Some("2027-01-01"));
        let out = serde_yaml::to_string(&c).expect("serialize");
        assert!(out.contains("revalidate_by"), "{out}");
    }

    #[test]
    fn round_trips_every_spec_field() {
        let c: Claim = serde_yaml::from_str(SPEC_SHAPED).expect("parse");
        let out = serde_yaml::to_string(&c).expect("serialize");
        for field in ["format", "id", "statement", "status", "evidence", "falsifier", "owner", "changed", "depends_on"] {
            assert!(out.contains(field), "round-trip dropped `{field}`:\n{out}");
        }
    }
}
