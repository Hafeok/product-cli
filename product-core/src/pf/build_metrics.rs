//! Build session metrics — tokens, rounds, files, verdict for one deliverable build.
//!
//! A pure, serializable record of what a `product build` run cost: every model
//! call (capability, gate, token usage), the files it touched, and the final
//! `done` verdict. The CLI collects calls into this during a build, then
//! persists + summarizes it (`.product/build/<id>.session.json`), so a feature
//! slice's implementation has an auditable cost + outcome record.

use std::collections::{BTreeMap, HashSet};

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

/// Depth and token metrics for the spec under construction.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SpecDepth {
    pub nodes: u64,
    pub depth: u64,
    pub acceptance: u64,
    pub deciders: u64,
    pub context_tokens: u64,
}

/// The metrics record for one build of one deliverable.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BuildSession {
    pub deliverable: String,
    pub elapsed_secs: u64,
    pub calls: Vec<CallRecord>,
    pub files: Vec<FileChange>,
    pub verdict: Verdict,
    pub spec_depth: SpecDepth,
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

    /// Returns true exactly when the calls used more than one distinct capability.
    pub fn escalated(&self) -> bool {
        let mut caps = HashSet::new();
        for c in &self.calls {
            caps.insert(&c.capability);
        }
        caps.len() > 1
    }

    /// Returns the gate that occurs in the most calls, or None when there are no calls.
    pub fn busiest_gate(&self) -> Option<String> {
        if self.calls.is_empty() {
            return None;
        }
        let mut counts = BTreeMap::new();
        for c in &self.calls {
            *counts.entry(c.gate.clone()).or_insert(0) += 1;
        }
        counts.into_iter().max_by_key(|(_, count)| *count).map(|(gate, _)| gate.clone())
    }
}

#[cfg(test)]
#[path = "build_metrics_tests.rs"]
mod tests;
