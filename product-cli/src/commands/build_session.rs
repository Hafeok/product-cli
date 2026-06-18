//! Build session collection — accumulate model calls + outcome during a build.
//!
//! A per-process collector (the build runs as one process) that the worker and
//! the fix loops feed: every model call records its capability, gate, and token
//! usage; the orchestrator sets the files + verdict at the end. `finish` returns
//! the pure [`BuildSession`] for the CLI to persist + summarize. Poisoned locks
//! degrade to no-ops rather than panic — metrics never break a build.

use std::sync::Mutex;
use std::time::Instant;

use product_core::pf::build_metrics::{BuildSession, CallRecord, FileChange, Verdict};

static SESSION: Mutex<Option<BuildSession>> = Mutex::new(None);
static GATE: Mutex<String> = Mutex::new(String::new());
static STARTED: Mutex<Option<Instant>> = Mutex::new(None);

/// Start a session for a deliverable build (initial gate: `dispatch`).
pub(super) fn begin(deliverable: &str) {
    if let Ok(mut s) = SESSION.lock() {
        *s = Some(BuildSession::new(deliverable));
    }
    if let Ok(mut t) = STARTED.lock() {
        *t = Some(Instant::now());
    }
    set_gate("dispatch");
}

/// Set the pipeline stage subsequent calls are attributed to.
pub(super) fn set_gate(gate: &str) {
    if let Ok(mut g) = GATE.lock() {
        *g = gate.to_string();
    }
}

/// Record one model call's token usage against the current gate.
pub(super) fn record_call(capability: &str, prompt_tokens: u64, completion_tokens: u64) {
    let gate = GATE.lock().map(|g| g.clone()).unwrap_or_default();
    if let Ok(mut guard) = SESSION.lock() {
        if let Some(s) = guard.as_mut() {
            s.calls.push(CallRecord { capability: capability.to_string(), gate, prompt_tokens, completion_tokens });
        }
    }
}

/// Total tokens spent so far this build — for budget enforcement mid-loop.
pub(super) fn tokens_spent() -> u64 {
    SESSION.lock().ok().and_then(|g| g.as_ref().map(|s| s.total_tokens())).unwrap_or(0)
}

/// Whether a token `budget` (if set) has been reached — stops escalation.
pub(super) fn over_budget(budget: Option<u64>) -> bool {
    matches!(budget, Some(b) if tokens_spent() >= b)
}

/// Record a call from a chat-completion response, reading its `usage` block.
pub(super) fn record_usage(capability: &str, response: &serde_json::Value) {
    let pt = response["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
    let ct = response["usage"]["completion_tokens"].as_u64().unwrap_or(0);
    record_call(capability, pt, ct);
}

/// Record the files the build touched.
pub(super) fn set_files(files: Vec<FileChange>) {
    if let Ok(mut guard) = SESSION.lock() {
        if let Some(s) = guard.as_mut() {
            s.files = files;
        }
    }
}

/// Close the session, stamping the verdict + elapsed time. Returns the record
/// (None when no session is active, e.g. a standalone `worker run`).
pub(super) fn finish(verdict: Verdict) -> Option<BuildSession> {
    let elapsed = STARTED.lock().ok().and_then(|t| *t).map(|i| i.elapsed().as_secs()).unwrap_or(0);
    let mut guard = SESSION.lock().ok()?;
    let mut s = guard.take()?;
    s.verdict = verdict;
    s.elapsed_secs = elapsed;
    Some(s)
}
