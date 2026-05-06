//! Shared envelope helpers for FT-059 health-check tools.

use crate::drift;
use serde_json::{json, Value};

/// Default drift source roots and ignore lists (mirrors the CLI handler).
pub(super) fn drift_source_settings() -> (Vec<String>, Vec<String>) {
    (
        vec!["src".to_string(), "crates".to_string()],
        vec![
            "target".to_string(),
            ".git".to_string(),
            "node_modules".to_string(),
        ],
    )
}

/// Build a summary block from a list of findings.
pub(super) fn summarize(findings: &[drift::DriftFinding]) -> Value {
    let mut high = 0u64;
    let mut medium = 0u64;
    let mut low = 0u64;
    let mut suppressed = 0u64;
    for f in findings {
        if f.suppressed {
            suppressed += 1;
            continue;
        }
        match f.severity {
            drift::DriftSeverity::High => high += 1,
            drift::DriftSeverity::Medium => medium += 1,
            drift::DriftSeverity::Low => low += 1,
        }
    }
    json!({
        "high": high,
        "medium": medium,
        "low": low,
        "suppressed": suppressed,
    })
}

/// Status enum: "clean" if no active findings, "findings" otherwise.
pub(super) fn status_for(findings: &[drift::DriftFinding]) -> &'static str {
    if findings.iter().any(|f| !f.suppressed) {
        "findings"
    } else {
        "clean"
    }
}

/// Encode a structured health-check error as a single JSON-RPC error
/// message. The first line is human-readable; subsequent lines carry the
/// JSON payload so callers can parse `tc_ids`, `id`, etc.
pub(super) fn health_error(code: &str, slug: &str, detail: Value) -> String {
    let payload = json!({
        "code": code,
        "kind": slug,
        "detail": detail,
    });
    let payload_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    format!("error[{}]: {}\n{}", code, slug, payload_str)
}
