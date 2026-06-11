//! Two Pillars conformance slice — clause checks over the knowledge graph (ADR-052).
//!
//! The Two Pillars specification defines what a What specification, a How
//! specification, and their derivation links MUST contain. In Product's
//! instantiation, features are the What pillar, ADRs are the How pillar,
//! and TCs are the declared acceptance criteria. This slice evaluates the
//! graph against the mechanically checkable subset of the Level 3
//! (spec-driven) clause set and reports per-clause outcomes.

mod check;
mod render;
#[cfg(test)]
mod tests;

pub use check::check;
pub use render::render_report_text;

use serde::Serialize;

/// Spec identifier reported in every conformance run.
pub const SPEC_ID: &str = "two-pillars/0.1";

/// Requirement strength of a finding, mapped from the spec's keywords:
/// a violated MUST is a violation; a disregarded SHOULD is an advisory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClauseSeverity {
    Violation,
    Advisory,
}

impl std::fmt::Display for ClauseSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Violation => write!(f, "violation"),
            Self::Advisory => write!(f, "advisory"),
        }
    }
}

/// How a clause is evaluated: `Checked` clauses scan artifacts and may
/// produce findings; `ByConstruction` clauses are guaranteed by the graph
/// loader or artifact model itself and always pass once the graph loads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClauseMode {
    Checked,
    ByConstruction,
}

/// One conformance finding against a specific clause.
#[derive(Debug, Clone, Serialize)]
pub struct ConformanceFinding {
    /// Stable clause identifier from the spec, e.g. `SPEC-WHAT-5`.
    pub clause: String,
    pub severity: ClauseSeverity,
    /// Artifact the finding points at, when the clause is per-artifact.
    pub artifact: Option<String>,
    pub description: String,
    pub suggested_action: String,
}

/// Per-clause outcome row for the report.
#[derive(Debug, Clone, Serialize)]
pub struct ClauseOutcome {
    pub clause: String,
    pub title: String,
    pub mode: ClauseMode,
    pub passed: bool,
    pub findings: usize,
}

/// Aggregate counts for a conformance run.
#[derive(Debug, Clone, Serialize)]
pub struct ConformanceSummary {
    pub clauses_checked: usize,
    pub clauses_passed: usize,
    pub violations: usize,
    pub advisories: usize,
}

/// Result of evaluating the graph against the checkable clause subset.
#[derive(Debug, Clone, Serialize)]
pub struct ConformanceReport {
    pub spec: String,
    /// `level-3` when no MUST clause is violated, else `below-level-3`.
    /// Only the mechanically checkable subset is evaluated — see
    /// `docs/two-pillars-conformance.md` for the full clause mapping.
    pub profile: String,
    pub scope: String,
    pub clauses: Vec<ClauseOutcome>,
    pub findings: Vec<ConformanceFinding>,
    pub summary: ConformanceSummary,
}

impl ConformanceReport {
    /// Whether any MUST clause is violated (advisories do not count).
    pub fn has_violations(&self) -> bool {
        self.summary.violations > 0
    }
}

/// Project-level declarations the clause checks need, extracted from the
/// repository config by the adapter so the slice stays I/O-free.
#[derive(Debug, Clone, Default)]
pub struct ProjectDeclarations {
    /// `name` from product config — the declared system identity (SPEC-WHAT-1).
    pub name: String,
    /// `[product].responsibility` — the declared purpose (SPEC-WHAT-2).
    pub responsibility: Option<String>,
    /// Configured What artifact path (features).
    pub features_path: String,
    /// Configured How artifact path (ADRs).
    pub adrs_path: String,
}
