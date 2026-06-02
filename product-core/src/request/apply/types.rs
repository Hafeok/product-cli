//! Apply pipeline result, option, payload types (FT-041, ADR-038, FT-064).
//!
//! Extracted from `apply/mod.rs` to keep the orchestrator under the 400-line
//! file-size fitness limit. The shapes are the public contract of
//! `apply_request` — `ApplyResult` round-trips into the MCP response.

use super::super::types::Finding;
use serde::Serialize;

#[derive(Default)]
pub struct ApplyOptions {
    /// Never write files — validate only.
    pub dry_run: bool,
    /// Skip git identity check (used by tests and migration).
    pub skip_git_identity: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatedArtifact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,
    pub id: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangedArtifact {
    pub id: String,
    pub mutations: usize,
    pub file: String,
}

/// One artifact removed by a `type: delete` request (FT-064).
#[derive(Debug, Clone, Serialize)]
pub struct DeletedArtifact {
    pub id: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyResult {
    pub applied: bool,
    pub created: Vec<CreatedArtifact>,
    pub changed: Vec<ChangedArtifact>,
    /// FT-064 — artifacts removed by a `type: delete` request.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deleted: Vec<DeletedArtifact>,
    pub findings: Vec<Finding>,
    pub graph_check_clean: bool,
    /// FT-053 / ADR-045 — features whose `started` tag was created as part
    /// of this apply (first `planned → in-progress` transition, or direct
    /// creation with `status: in-progress`). Empty when git is unavailable.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub started_tags: Vec<String>,
    /// FT-053 / ADR-045 — W-class messages emitted when started-tag creation
    /// was skipped (git unavailable) or failed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub started_tag_warnings: Vec<String>,
}

impl ApplyResult {
    #[allow(dead_code)]
    pub fn errors(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.is_error()).collect()
    }
    #[allow(dead_code)]
    pub fn warnings(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| !f.is_error()).collect()
    }
}
