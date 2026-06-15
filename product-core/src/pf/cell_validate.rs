//! Task-type (cell) conformance checker, cross-validated against What + How.
//!
//! Beyond the §5 structural rules — slots and audits present, **no slot
//! without a backing audit** — this resolves each cell's frozen-input pointers:
//! `domain:X` must be a declared domain slot or a real node in the captured
//! What graph; bare pointers must name another cell or slot; `applies` must
//! reference a pattern/principle in the How contract. Dangling cross-references
//! are warnings; the load-bearing structural rules are violations.

use std::collections::BTreeSet;

use super::cell::TaskType;
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

/// Validate a task type. `domain`/`how` enable cross-graph resolution when
/// available; without them, domain/pattern pointers are reported as warnings.
pub fn validate_cell(t: &TaskType, domain: Option<&DomainGraph>, how: Option<&HowContract>) -> Vec<Violation> {
    let mut out = Vec::new();
    if t.slots.is_empty() {
        out.push(v(&t.id, "slots", "§5 A task type must declare at least one dual-read slot."));
    }
    if t.audits.is_empty() {
        out.push(v(&t.id, "audits", "§6.1 A task type must declare at least one audit (name what it protects)."));
    }
    check_slot_coverage(t, &mut out);
    check_cells(t, domain, &mut out);
    check_applies(t, how, &mut out);
    out
}

/// §5/§6.1 — no slot without a backing audit (its required inline `audit`
/// field must be non-empty), and every audit must name what it protects.
/// A slot not also referenced by a top-level audit's `protects` is a soft
/// coverage warning, not a violation.
fn check_slot_coverage(t: &TaskType, out: &mut Vec<Violation>) {
    for slot in &t.slots {
        if slot.audit.trim().is_empty() {
            out.push(v(&t.id, "slots",
                &format!("§5/§6.1 Slot '{}' declares no audit — no slot without a backing audit.", slot.name)));
        } else if !t.audits.iter().any(|a| a.protects.contains(&slot.name)) {
            out.push(warn(&t.id, "audits",
                &format!("Slot '{}' is not named by any top-level audit's `protects` — consider adding one.", slot.name)));
        }
    }
    for a in &t.audits {
        if a.protects.trim().is_empty() {
            out.push(v(&a.id, "protects", "§6.1 An audit must name what it protects."));
        }
    }
}

/// Resolve each cell's `derived_from` pointer against slots, sibling cells, the
/// What graph, or recognised external prefixes.
fn check_cells(t: &TaskType, domain: Option<&DomainGraph>, out: &mut Vec<Violation>) {
    let slots: BTreeSet<&str> = t.slots.iter().map(|s| s.name.as_str()).collect();
    let cells: BTreeSet<&str> = t.cells.iter().map(|c| c.id.as_str()).collect();
    for cell in &t.cells {
        if cell.derived_from.is_empty() {
            out.push(v(&cell.id, "derived_from",
                "§5 A cell must declare its frozen input (derived_from)."));
        }
        for ptr in &cell.derived_from {
            if let Some(msg) = unresolved(ptr, &slots, &cells, domain) {
                out.push(warn(&cell.id, "derived_from", &msg));
            }
        }
    }
}

/// Return a warning message if `ptr` resolves to nothing, else `None`.
fn unresolved(ptr: &str, slots: &BTreeSet<&str>, cells: &BTreeSet<&str>, domain: Option<&DomainGraph>) -> Option<String> {
    if let Some(rest) = ptr.strip_prefix("domain:") {
        let ok = slots.contains(rest) || domain.map(|g| g.contains(rest)).unwrap_or(false);
        return (!ok).then(|| format!(
            "derived_from 'domain:{rest}' resolves to neither a declared domain slot nor a node in the What graph"
        ));
    }
    if let Some(rest) = ptr.strip_prefix("slot:").or_else(|| ptr.strip_prefix("behaviour:")) {
        return (!slots.contains(rest)).then(|| format!("derived_from '{ptr}' names no declared slot"));
    }
    if ptr.contains(':') {
        return None; // app-contract:/infra:/pattern:/… — cross-file, accepted
    }
    // A bare token must name a sibling cell or a slot.
    (!cells.contains(ptr) && !slots.contains(ptr))
        .then(|| format!("derived_from '{ptr}' matches no sibling cell or slot"))
}

/// Each `applies` pointer should name a pattern or principle in the How
/// contract; without a How contract, surface them as warnings to confirm.
fn check_applies(t: &TaskType, how: Option<&HowContract>, out: &mut Vec<Violation>) {
    let known: Option<BTreeSet<&str>> = how.map(|h| {
        h.patterns.iter().map(|p| p.id.as_str())
            .chain(h.principles.iter().map(|p| p.id.as_str()))
            .collect()
    });
    for cell in &t.cells {
        for ap in &cell.applies {
            let unknown = match &known {
                Some(set) => !set.contains(ap.as_str()),
                None => true,
            };
            if unknown {
                let detail = if known.is_some() { "not a pattern/principle in the How contract" } else { "no How contract loaded to confirm it" };
                out.push(warn(&cell.id, "applies", &format!("cell applies '{ap}' — {detail}")));
            }
        }
    }
}

#[cfg(test)]
#[path = "cell_validate_tests.rs"]
mod tests;
