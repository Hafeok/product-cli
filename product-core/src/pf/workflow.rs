//! Workflow session — the journey journal for a What → How → Build run.
//!
//! Tracks where a guided lifecycle session currently is (its phase), how far it
//! may go (`until`), and the transitions it has made — distinct from the What
//! graph itself, which lives in a draft [`super::session::DomainSession`]. The
//! phase-gated MCP server loads this record per call to decide which tools are
//! callable, and the session-control tools mutate + persist it. Pure library —
//! no MCP, no CLI: the agent CLI is stored as a plain string.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::build_metrics::Verdict;
use crate::error::{ProductError, Result};

/// The lifecycle phase, ordered `What < How < Build`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    What,
    How,
    Build,
}

impl Phase {
    /// The phase reached by a single forward step, or `None` past `Build`.
    pub fn next(self) -> Option<Phase> {
        match self {
            Phase::What => Some(Phase::How),
            Phase::How => Some(Phase::Build),
            Phase::Build => None,
        }
    }

    /// The canonical lowercase name (matches the serde representation).
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::What => "what",
            Phase::How => "how",
            Phase::Build => "build",
        }
    }

    /// Parse from a flag/string value (case-insensitive).
    pub fn parse(s: &str) -> Result<Phase> {
        match s.trim().to_lowercase().as_str() {
            "what" => Ok(Phase::What),
            "how" => Ok(Phase::How),
            "build" => Ok(Phase::Build),
            other => Err(ProductError::ConfigError(format!(
                "unknown phase: {other}\n  = hint: use `what`, `how`, or `build`"
            ))),
        }
    }

    /// All phases in order.
    pub fn all() -> [Phase; 3] {
        [Phase::What, Phase::How, Phase::Build]
    }
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// One recorded phase transition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseEvent {
    pub from: Phase,
    pub to: Phase,
    pub at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The persisted state of one What → How → Build session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowSession {
    pub id: String,
    pub product: String,
    pub phase: Phase,
    /// The hosting agent CLI (`claude` | `copilot`); stored as a string to keep
    /// this module free of the `crate::author` dependency.
    pub agent_cli: String,
    /// The furthest phase this session may advance to (the `--until` cap).
    pub until: Phase,
    pub started_at: String,
    #[serde(default)]
    pub history: Vec<PhaseEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_verdict: Option<Verdict>,
    #[serde(default)]
    pub finalized: bool,
}

/// Where a session's workflow journal is stored (distinct from the draft graph's
/// `session.json`, which shares the directory).
pub fn workflow_path(session_dir: &Path) -> PathBuf {
    session_dir.join("workflow.json")
}

impl WorkflowSession {
    /// Begin a new session at the `What` phase. `now` is an RFC-3339 timestamp.
    pub fn new(id: &str, product: &str, agent_cli: &str, until: Phase, now: String) -> Self {
        Self {
            id: id.to_string(),
            product: product.to_string(),
            phase: Phase::What,
            agent_cli: agent_cli.to_string(),
            until,
            started_at: now,
            history: Vec::new(),
            build_verdict: None,
            finalized: false,
        }
    }

    /// Load the journal from a session directory.
    pub fn load(session_dir: &Path) -> Result<Self> {
        let path = workflow_path(session_dir);
        let text = std::fs::read_to_string(&path)
            .map_err(|_| ProductError::NotFound(format!("no workflow session at {}", path.display())))?;
        serde_json::from_str(&text)
            .map_err(|e| ProductError::Internal(format!("corrupt workflow session: {e}")))
    }

    /// Persist the journal atomically.
    pub fn save(&self, session_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(session_dir).map_err(|e| ProductError::WriteError {
            path: session_dir.to_path_buf(),
            message: e.to_string(),
        })?;
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| ProductError::Internal(format!("serialize workflow session: {e}")))?;
        crate::fileops::write_file_atomic(&workflow_path(session_dir), &text)
    }

    /// Advance the phase. With `to = None` it steps to the next phase; with a
    /// target it jumps forward to that phase. The move must be strictly forward
    /// and may not pass the `until` cap. Records a [`PhaseEvent`].
    pub fn advance(&mut self, to: Option<Phase>, now: String) -> Result<Phase> {
        let target = match to {
            Some(p) => p,
            None => self.phase.next().ok_or_else(|| {
                ProductError::ConfigError(format!("already at the final phase ({})", self.phase))
            })?,
        };
        if target <= self.phase {
            return Err(ProductError::ConfigError(format!(
                "cannot advance from {} to {} — phases only move forward",
                self.phase, target
            )));
        }
        if target > self.until {
            return Err(ProductError::ConfigError(format!(
                "cannot advance to {} — this session is capped at {}",
                target, self.until
            )));
        }
        let from = self.phase;
        self.phase = target;
        self.history.push(PhaseEvent { from, to: target, at: now, note: None });
        Ok(target)
    }

    /// Record the outcome of a build run.
    pub fn record_build(&mut self, verdict: Verdict) {
        self.build_verdict = Some(verdict);
    }

    /// A summary for `workflow_status` (callers add the available-tool list).
    pub fn status_json(&self) -> Value {
        json!({
            "id": self.id,
            "product": self.product,
            "phase": self.phase,
            "until": self.until,
            "nextPhase": self.phase.next().filter(|p| *p <= self.until),
            "startedAt": self.started_at,
            "history": self.history,
            "buildVerdict": self.build_verdict,
            "finalized": self.finalized,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_orders_what_how_build() {
        assert!(Phase::What < Phase::How);
        assert!(Phase::How < Phase::Build);
        assert_eq!(Phase::What.next(), Some(Phase::How));
        assert_eq!(Phase::Build.next(), None);
    }

    #[test]
    fn parse_round_trips_names() {
        for p in Phase::all() {
            assert_eq!(Phase::parse(p.as_str()).expect("parse"), p);
        }
        assert!(Phase::parse("nope").is_err());
    }

    #[test]
    fn advance_steps_to_next_phase() {
        let mut s = WorkflowSession::new("s1", "demo", "claude", Phase::Build, "t0".into());
        assert_eq!(s.advance(None, "t1".into()).expect("advance"), Phase::How);
        assert_eq!(s.phase, Phase::How);
        assert_eq!(s.history.len(), 1);
        assert_eq!(s.history[0].from, Phase::What);
        assert_eq!(s.history[0].to, Phase::How);
    }

    #[test]
    fn advance_can_jump_forward() {
        let mut s = WorkflowSession::new("s1", "demo", "claude", Phase::Build, "t0".into());
        assert_eq!(s.advance(Some(Phase::Build), "t1".into()).expect("jump"), Phase::Build);
        assert_eq!(s.phase, Phase::Build);
    }

    #[test]
    fn advance_respects_until_cap() {
        let mut s = WorkflowSession::new("s1", "demo", "claude", Phase::What, "t0".into());
        let err = s.advance(None, "t1".into()).unwrap_err();
        assert!(format!("{err}").contains("capped"));
        assert_eq!(s.phase, Phase::What);
    }

    #[test]
    fn advance_refuses_to_go_backward() {
        let mut s = WorkflowSession::new("s1", "demo", "claude", Phase::Build, "t0".into());
        s.advance(Some(Phase::Build), "t1".into()).expect("jump");
        assert!(s.advance(Some(Phase::What), "t2".into()).is_err());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut s = WorkflowSession::new("s1", "demo", "claude", Phase::Build, "t0".into());
        s.advance(None, "t1".into()).expect("advance");
        s.record_build(Verdict { done: true, passing: 3, total: 3 });
        s.save(dir.path()).expect("save");
        let loaded = WorkflowSession::load(dir.path()).expect("load");
        assert_eq!(loaded, s);
        assert_eq!(loaded.phase, Phase::How);
        assert!(loaded.build_verdict.expect("verdict").done);
    }

    #[test]
    fn load_missing_is_clear() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = WorkflowSession::load(dir.path()).unwrap_err();
        assert!(format!("{err}").contains("no workflow session"));
    }
}
