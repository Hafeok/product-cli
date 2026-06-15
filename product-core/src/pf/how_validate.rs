//! How-contract conformance checker mirroring the framework How shapes.
//!
//! Each rule corresponds to a shape in `schema/shapes/how.shacl.ttl`, carrying
//! the same framework-section message — including the crown rule (the rationale
//! trace must be true). Validates the file model directly; the Turtle
//! projection of a contract that passes here also passes `how.shacl.ttl`.

use std::collections::HashSet;

use super::how::HowContract;
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

/// True if any entry is a blocking violation (not a warning).
pub fn has_blocking(results: &[Violation]) -> bool {
    results.iter().any(|r| r.severity == "violation")
}

/// Run every How shape over the contract; returns all blocking violations.
pub fn validate_how(c: &HowContract) -> Vec<Violation> {
    let mut out = Vec::new();
    let principle_ids: HashSet<&str> = c.principles.iter().map(|p| p.id.as_str()).collect();
    let pattern_ids: HashSet<&str> = c.patterns.iter().map(|p| p.id.as_str()).collect();
    let applied = applied_principles(c);

    check_decisions(c, &principle_ids, &mut out);
    check_principles(c, &pattern_ids, &applied, &mut out);
    check_patterns(c, &principle_ids, &mut out);
    check_app_contract(c, &mut out);
    check_infra_contract(c, &mut out);
    check_interfaces(c, &mut out);
    out
}

/// Principle ids that are "applied": realised by a pattern that some cell or
/// task-type applies (`applied_by` non-empty) — the trace's left side.
fn applied_principles(c: &HowContract) -> HashSet<String> {
    let mut set = HashSet::new();
    for pat in &c.patterns {
        if !pat.applied_by.is_empty() {
            for pid in &pat.realizes {
                set.insert(pid.clone());
            }
        }
    }
    set
}

fn check_decisions(c: &HowContract, principles: &HashSet<&str>, out: &mut Vec<Violation>) {
    for d in &c.top_decisions {
        if d.rationale.trim().is_empty() {
            out.push(v(&d.id, "rationale",
                "§4.1 A top decision must carry rationale (what it buys, when it applies, when it does not)."));
        }
        for lic in &d.licenses {
            if !principles.contains(lic.as_str()) {
                out.push(warn(&d.id, "licenses",
                    "§4.1 A top decision's licenses should each reference a defined Principle."));
            }
        }
    }
}

fn check_principles(c: &HowContract, patterns: &HashSet<&str>, applied: &HashSet<String>, out: &mut Vec<Violation>) {
    for p in &c.principles {
        if p.statement.trim().is_empty() {
            out.push(v(&p.id, "statement",
                "§4.1 A principle must be stated checkably (a vague principle cannot back a verification)."));
        }
        let earns = !p.enforced_by.is_empty() || applied.contains(&p.id);
        if !earns {
            out.push(v(&p.id, "earn-their-place",
                "§4.1 Earn-their-place: a principle must be applied by a work unit or enforced by a verification, else it is documentation, not architecture."));
        }
        // The crown rule: an applied principle must be enforced.
        if applied.contains(&p.id) && p.enforced_by.is_empty() {
            out.push(v(&p.id, "trace",
                "§5/§4.1 The trace must be true: every principle a work unit applies must be enforced by a passing verification, or the claim must be retracted."));
        }
        for r in &p.realized_by {
            if !patterns.contains(r.as_str()) {
                out.push(warn(&p.id, "realized_by",
                    "§4.1 A principle's realized_by should reference a defined Pattern."));
            }
        }
    }
}

fn check_patterns(c: &HowContract, principles: &HashSet<&str>, out: &mut Vec<Violation>) {
    for p in &c.patterns {
        if p.realizes.is_empty() {
            out.push(v(&p.id, "realizes",
                "§4.1 A pattern must realise at least one principle (it is how a principle is concretely met)."));
        }
        for r in &p.realizes {
            if !principles.contains(r.as_str()) {
                out.push(v(&p.id, "realizes",
                    "§4.1 A pattern's realizes must reference a Principle that exists."));
            }
        }
    }
}

fn check_app_contract(c: &HowContract, out: &mut Vec<Violation>) {
    let a = &c.application_contract;
    if a.id.trim().is_empty() || a.language.trim().is_empty() {
        out.push(v(&a.id, "label", "§4.2 An application contract must be named (id + language)."));
    }
    if a.statements.is_empty() {
        out.push(v(&a.id, "statements",
            "§4.2 An application contract must carry at least one checkable statement (a convention that cannot be checked cannot be a contract)."));
    }
    for s in &a.statements {
        if s.statement.trim().is_empty() {
            out.push(v(&s.id, "statement", "§4.2 A contract statement must be a checkable assertion."));
        }
    }
}

fn check_infra_contract(c: &HowContract, out: &mut Vec<Violation>) {
    if let Some(infra) = &c.infrastructure_contract {
        if infra.satisfies != c.application_contract.id {
            out.push(v(&infra.id, "conformsTo",
                "§4.2 An infrastructure contract must satisfy (conformsTo) the application contract."));
        }
    }
}

fn check_interfaces(c: &HowContract, out: &mut Vec<Violation>) {
    for i in &c.interface_contracts {
        if i.derived_from.is_empty() {
            out.push(v(&i.id, "derivedFrom",
                "§4.4 An interface contract must be generated from (derivedFrom) the domain/event model, never hand-written."));
        }
    }
}

#[cfg(test)]
#[path = "how_validate_tests.rs"]
mod tests;
