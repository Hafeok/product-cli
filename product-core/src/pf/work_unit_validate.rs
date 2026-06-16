//! Work-unit (SPMC) checker, cross-validated against What + How.
//!
//! Structural §5 rules — a frozen, non-empty context, a prompt, a schema, and
//! exactly one produced artifact — plus cross-checks: a `domain:X` input and
//! `trace.what` resolve to real entities in the captured What graph; `applies`
//! and `trace.why` name real How patterns/principles; an applied principle
//! should be enforced (the crown trace-truth, surfaced as a warning here since
//! the verification graph is separate). Structural breaks are violations;
//! cross-reference gaps are warnings.

use std::collections::BTreeSet;

use super::how::HowContract;
use super::model::DomainGraph;
use super::validate::Violation;

fn v(focus: &str, path: &str, message: &str) -> Violation {
    sev(focus, path, message, "violation")
}
fn warn(focus: &str, path: &str, message: &str) -> Violation {
    sev(focus, path, message, "warning")
}
fn sev(focus: &str, path: &str, message: &str, severity: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: severity.to_string(),
    }
}

/// Validate a work unit. `domain`/`how` enable cross-graph resolution.
pub fn validate_work_unit(w: &super::work_unit::WorkUnit, domain: Option<&DomainGraph>, how: Option<&HowContract>) -> Vec<Violation> {
    let mut out = Vec::new();
    if w.schema.trim().is_empty() {
        out.push(v(&w.id, "schema", "§5 A work unit must declare its output schema (S of SPMC)."));
    }
    if w.prompt.trim().is_empty() {
        out.push(v(&w.id, "prompt", "§5 A work unit must declare its single-purpose prompt (P of SPMC)."));
    }
    if !w.context.frozen {
        out.push(v(&w.id, "context.frozen",
            "§5 A work unit's context must be frozen — reproducibility depends on the same input yielding the same output."));
    }
    if w.context.derived_from.is_empty() {
        out.push(v(&w.id, "context.derived_from",
            "§5 A work unit must declare its frozen input (what it is derived_from)."));
    }
    if w.produces.artifact.trim().is_empty() {
        out.push(v(&w.id, "produces.artifact", "§5 A work unit must produce exactly one named artifact."));
    }
    check_domain_refs(w, domain, &mut out);
    check_applies(w, how, &mut out);
    out
}

fn check_domain_refs(w: &super::work_unit::WorkUnit, domain: Option<&DomainGraph>, out: &mut Vec<Violation>) {
    for ptr in &w.context.derived_from {
        if let Some(rest) = ptr.strip_prefix("domain:") {
            if !domain.map(|g| g.contains(rest)).unwrap_or(true) {
                out.push(warn(&w.id, "context.derived_from",
                    &format!("'domain:{rest}' is not a node in the captured What graph")));
            }
        }
    }
    if let Some(what) = w.trace.as_ref().and_then(|t| t.what.as_deref()) {
        if let Some(g) = domain {
            if !g.contains(what) {
                out.push(warn(&w.id, "trace.what",
                    &format!("trace.what '{what}' is not a node in the captured What graph")));
            }
        }
    }
}

fn check_applies(w: &super::work_unit::WorkUnit, how: Option<&HowContract>, out: &mut Vec<Violation>) {
    let known: Option<BTreeSet<&str>> = how.map(|h| {
        h.patterns.iter().map(|p| p.id.as_str())
            .chain(h.principles.iter().map(|p| p.id.as_str()))
            .collect()
    });
    let enforced: Option<BTreeSet<&str>> = how.map(|h| {
        h.principles.iter().filter(|p| !p.enforced_by.is_empty()).map(|p| p.id.as_str())
            .chain(h.patterns.iter().filter(|p| !p.enforced_by.is_empty()).map(|p| p.id.as_str()))
            .collect()
    });
    for ap in w.applies.iter().chain(w.trace.iter().flat_map(|t| t.why.iter())) {
        match &known {
            Some(set) if !set.contains(ap.as_str()) =>
                out.push(warn(&w.id, "applies", &format!("applies '{ap}' — not a pattern/principle in the How contract"))),
            Some(_) => {
                // known: check the crown trace-truth — it must be enforced.
                if let Some(e) = &enforced {
                    if !e.contains(ap.as_str()) {
                        out.push(warn(&w.id, "trace",
                            &format!("applies '{ap}' but the How does not record it as enforced_by a verification (the trace must be true)")));
                    }
                }
            }
            None => out.push(warn(&w.id, "applies", &format!("applies '{ap}' — no How contract loaded to confirm it"))),
        }
    }
}

#[cfg(test)]
#[path = "work_unit_validate_tests.rs"]
mod tests;
