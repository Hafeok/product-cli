//! Domain-authoring session state container.
//!
//! Holds the in-progress What graph plus the provenance a finalized session
//! needs (product, participants, derivation length). Persisted as JSON so a
//! stdio MCP server can reload it per call; the Turtle export is produced
//! only at `session_finalize`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::model::DomainGraph;
use super::provenance::{content_hash, Provenance};
use super::questions::{open_questions, Focus};
use super::turtle::to_turtle;
use super::validate::{validate_graph, Violation};
use super::ids::validate_id;
use crate::error::{ProductError, Result};

/// The persisted state of one What-capture session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomainSession {
    pub product: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub participants: Vec<String>,
    #[serde(default)]
    pub graph: DomainGraph,
    #[serde(default)]
    pub tool_calls: usize,
    pub started_at: String,
    #[serde(default)]
    pub finalized: bool,
}

/// The result of `session_finalize`: violations if non-conformant, else the
/// exported Turtle plus the provenance record.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Finalized {
    pub ok: bool,
    pub violations: Vec<Violation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turtle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

/// Where the single active session for a server is stored.
pub fn session_path(session_dir: &Path) -> PathBuf {
    session_dir.join("session.json")
}

impl DomainSession {
    /// Open or create a session. `now` is the caller-supplied timestamp
    /// (RFC-3339); `seed_graph` optionally seeds from prior Turtle.
    pub fn start(
        product: &str,
        title: Option<String>,
        participants: Vec<String>,
        seed_graph: Option<&str>,
        now: String,
    ) -> Result<Self> {
        validate_id(product)?;
        let graph = match seed_graph {
            Some(ttl) if !ttl.trim().is_empty() => super::seed::from_turtle(ttl)?,
            _ => DomainGraph::default(),
        };
        Ok(Self { product: product.to_string(), title, participants, graph, tool_calls: 0, started_at: now, finalized: false })
    }

    /// Load the active session, or a clear error telling the model to start one.
    pub fn load(session_dir: &Path) -> Result<Self> {
        let path = session_path(session_dir);
        let text = std::fs::read_to_string(&path).map_err(|_| {
            ProductError::NotFound("no active session — call session_start first".to_string())
        })?;
        serde_json::from_str(&text)
            .map_err(|e| ProductError::Internal(format!("corrupt session file: {}", e)))
    }

    /// Persist the session atomically.
    pub fn save(&self, session_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(session_dir).map_err(|e| ProductError::WriteError {
            path: session_dir.to_path_buf(),
            message: e.to_string(),
        })?;
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| ProductError::Internal(format!("serialize session: {}", e)))?;
        crate::fileops::write_file_atomic(&session_path(session_dir), &text)
    }

    /// A summary for `session_state`: counts, conformance, open questions.
    pub fn state_json(&self) -> Value {
        let violations = validate_graph(&self.graph);
        let counts: serde_json::Map<String, Value> = self.graph.counts().into_iter()
            .map(|(k, n)| (k.to_string(), json!(n)))
            .collect();
        json!({
            "product": self.product,
            "title": self.title,
            "participants": self.participants,
            "toolCalls": self.tool_calls,
            "counts": counts,
            "nodeCount": self.graph.node_count(),
            "conformant": violations.is_empty(),
            "violations": violations,
            "openQuestions": open_questions(&self.graph, Focus::All),
        })
    }

    /// Full validation: every shape over the whole graph.
    pub fn validate_json(&self) -> Value {
        let violations = validate_graph(&self.graph);
        json!({ "conformant": violations.is_empty(), "violations": violations })
    }

    /// Run finalization. `now` is the finalize timestamp (RFC-3339). When
    /// non-conformant, returns the blocking violations without finalizing.
    pub fn finalize(&self, now: String) -> Finalized {
        let violations = validate_graph(&self.graph);
        if !violations.is_empty() {
            return Finalized { ok: false, violations, turtle: None, provenance: None };
        }
        let turtle = to_turtle(&self.graph, &self.product);
        let provenance = Provenance {
            product: self.product.clone(),
            title: self.title.clone(),
            participants: self.participants.clone(),
            content_hash: content_hash(&turtle),
            finalized_at: now,
            tool_call_count: self.tool_calls,
        };
        Finalized { ok: true, violations, turtle: Some(turtle), provenance: Some(provenance) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_rejects_bad_product_id() {
        assert!(DomainSession::start("1bad", None, vec![], None, "t".into()).is_err());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let s = DomainSession::start("demo", Some("Demo".into()), vec!["PO".into()], None, "t".into()).expect("start");
        s.save(dir.path()).expect("save");
        let loaded = DomainSession::load(dir.path()).expect("load");
        assert_eq!(loaded.product, "demo");
        assert_eq!(loaded.participants, vec!["PO".to_string()]);
    }

    #[test]
    fn load_without_session_is_clear() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = DomainSession::load(dir.path()).unwrap_err();
        assert!(format!("{}", err).contains("session_start"));
    }

    #[test]
    fn finalize_blocks_when_non_conformant() {
        let mut s = DomainSession::start("demo", None, vec![], None, "t".into()).expect("start");
        s.graph.events.push(super::super::model::Event { id: "E".into(), label: "E".into(), context: "Nope".into(), changes: "Nope".into() });
        let fin = s.finalize("t2".into());
        assert!(!fin.ok);
        assert!(fin.turtle.is_none());
        assert!(!fin.violations.is_empty());
    }
}
