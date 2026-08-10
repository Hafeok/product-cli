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
    /// Optional version index (format 4): the toolchain version the closure
    /// this claim reports is observed against — its drift anchor. A closure
    /// claim states what an arrangement closes, and an arrangement is a
    /// versioned thing; the index is what a promoting repo checks against
    /// its own toolchain before inheriting the entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_index: Option<VersionIndex>,
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

/// What toolchain a closure was observed against (format 4).
///
/// Every field is optional because the anchor differs by claim: a language
/// rule pins `language`, an SDK behaviour pins `sdk`, an analyzer finding
/// pins `tool`. At least one must be present — an index that names no
/// version anchors nothing, which `validate` reports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionIndex {
    /// Language version, e.g. `C# 12`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// SDK version the observation was made on, e.g. `8.0.413`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdk: Option<String>,
    /// Target framework or runtime, e.g. `net8.0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Tool version when the closure rests on a tool rather than the
    /// language — an analyzer package, a linter, a language server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

impl VersionIndex {
    /// An index must anchor something; all-empty is not an index.
    pub fn is_empty(&self) -> bool {
        self.language.is_none()
            && self.sdk.is_none()
            && self.target.is_none()
            && self.tool.is_none()
    }
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
    fn a_format_four_claim_may_index_the_toolchain_it_was_observed_on() {
        let text = SPEC_SHAPED.replace(
            "format: 1",
            "format: 4\nversion_index:\n  language: \"C# 12\"\n  sdk: 8.0.413\n  target: net8.0",
        );
        let c: Claim = serde_yaml::from_str(&text).expect("parse");
        let idx = c.version_index.as_ref().expect("index");
        assert_eq!(idx.language.as_deref(), Some("C# 12"));
        assert_eq!(idx.sdk.as_deref(), Some("8.0.413"));
        assert_eq!(idx.target.as_deref(), Some("net8.0"));
        assert_eq!(idx.tool, None);
        assert!(!idx.is_empty());
        let out = serde_yaml::to_string(&c).expect("serialize");
        assert!(out.contains("version_index"), "{out}");
        assert!(!out.contains("tool"), "absent anchors stay absent: {out}");
    }

    #[test]
    fn an_index_naming_no_version_anchors_nothing() {
        assert!(VersionIndex::default().is_empty());
        let anchored = VersionIndex { tool: Some("bicep-ls 0.30".into()), ..Default::default() };
        assert!(!anchored.is_empty());
    }

    #[test]
    fn a_claim_without_an_index_still_omits_the_field() {
        let c: Claim = serde_yaml::from_str(SPEC_SHAPED).expect("parse");
        assert!(c.version_index.is_none());
        let out = serde_yaml::to_string(&c).expect("serialize");
        assert!(!out.contains("version_index"), "{out}");
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
