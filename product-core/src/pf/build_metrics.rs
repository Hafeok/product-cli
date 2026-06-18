//! Build session metrics — tokens, rounds, files, verdict for one deliverable build.
//!
//! A pure, serializable record of what a `product build` run cost: every model
//! call (capability, gate, token usage), the files it touched, and the final
//! `done` verdict. The CLI collects calls into this during a build, then
//! persists + summarizes it (`.product/build/<id>.session.json`), so a feature
//! slice's implementation has an auditable cost + outcome record.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One model call made during a build.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CallRecord {
    pub capability: String,
    /// The pipeline stage: `dispatch`, `lsp`, or `verify`.
    pub gate: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

/// A file the build created or changed.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FileChange {
    pub path: String,
    pub status: String,
}

/// The computed `done` outcome at the end of the build.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Verdict {
    pub done: bool,
    pub passing: usize,
    pub total: usize,
}

/// The metrics record for one build of one deliverable.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BuildSession {
    pub deliverable: String,
    pub elapsed_secs: u64,
    pub calls: Vec<CallRecord>,
    pub files: Vec<FileChange>,
    pub verdict: Verdict,
}

impl BuildSession {
    pub fn new(deliverable: &str) -> Self {
        Self { deliverable: deliverable.to_string(), ..Default::default() }
    }

    pub fn prompt_tokens(&self) -> u64 {
        self.calls.iter().map(|c| c.prompt_tokens).sum()
    }

    pub fn completion_tokens(&self) -> u64 {
        self.calls.iter().map(|c| c.completion_tokens).sum()
    }

    pub fn total_tokens(&self) -> u64 {
        self.prompt_tokens() + self.completion_tokens()
    }

    /// Number of model calls per gate (`dispatch`/`lsp`/`verify`) — the rounds.
    pub fn rounds(&self) -> BTreeMap<String, usize> {
        let mut out = BTreeMap::new();
        for c in &self.calls {
            *out.entry(c.gate.clone()).or_insert(0) += 1;
        }
        out
    }

    /// Total tokens attributed to each capability.
    pub fn tokens_by_capability(&self) -> BTreeMap<String, u64> {
        let mut out = BTreeMap::new();
        for c in &self.calls {
            *out.entry(c.capability.clone()).or_insert(0) += c.prompt_tokens + c.completion_tokens;
        }
        out
    }

    pub fn to_json(&self) -> Result<String, crate::error::ProductError> {
        serde_json::to_string_pretty(self).map_err(|e| crate::error::ProductError::Internal(format!("serialize build session: {e}")))
    }
}

#[cfg(test)]
#[path = "build_metrics_tests.rs"]
mod tests;
