//! Data types for codebase onboarding (ADR-027)

use serde::{Deserialize, Serialize};

/// A piece of evidence grounding a decision candidate in source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub file: String,
    pub line: usize,
    pub snippet: String,
    #[serde(default = "default_evidence_valid")]
    pub evidence_valid: bool,
}

fn default_evidence_valid() -> bool {
    true
}

/// A decision candidate produced by the scan phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub signal_type: String,
    pub title: String,
    pub observation: String,
    pub evidence: Vec<Evidence>,
    pub hypothesised_consequence: String,
    pub confidence: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Metadata about a scan run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    pub files_scanned: usize,
    pub prompt_version: String,
}

/// The output of a scan phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOutput {
    pub candidates: Vec<Candidate>,
    pub scan_metadata: ScanMetadata,
}

/// Triage action for a candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriageAction {
    Confirm,
    Reject,
    Merge(String), // target candidate ID
    Skip,
}

/// Status of a candidate after triage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TriageStatus {
    Confirmed,
    Rejected,
    Merged,
    Skipped,
    Pending,
}

/// A triaged candidate — the original candidate plus triage metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriagedCandidate {
    #[serde(flatten)]
    pub candidate: Candidate,
    pub triage_status: TriageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_into: Option<String>,
}

/// Output of the triage phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageOutput {
    pub candidates: Vec<TriagedCandidate>,
}

/// A proposed feature stub from the seed phase.
#[derive(Debug, Clone)]
pub struct ProposedFeatureStub {
    pub id: String,
    pub title: String,
    pub adr_ids: Vec<String>,
    pub filename: String,
}

/// A proposed ADR from the seed phase.
#[derive(Debug, Clone)]
pub struct ProposedAdr {
    pub id: String,
    pub title: String,
    pub observation: String,
    pub evidence: Vec<Evidence>,
    pub hypothesised_consequence: String,
    pub filename: String,
}

/// Result of the seed phase.
#[derive(Debug, Clone)]
pub struct SeedResult {
    pub adrs: Vec<ProposedAdr>,
    pub features: Vec<ProposedFeatureStub>,
}
