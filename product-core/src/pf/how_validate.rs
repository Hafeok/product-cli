//! How-contract conformance checker mirroring the framework How shapes.
//!
//! Splits along the §6 line: presence and cardinality checks (a rationale must
//! be non-empty, a pattern must realise at least one principle) stay native
//! here, while every cross-reference and the crown trace rule (each `licenses`,
//! `realizes`, `conformsTo`, `realized_by` edge must resolve; every applied
//! principle must be enforced) are SPARQL rules run over the Turtle projection
//! by `sparql_rules` — the constraint lives in the graph, not a field-walk.

use super::how::HowContract;
use super::how_turtle::how_to_turtle;
use super::rules_how::how_rules;
use super::sparql_rules::run_rules;
use super::validate::Violation;

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: "violation".to_string(),
    }
}

/// True if any entry is a blocking violation (not a warning).
pub fn has_blocking(results: &[Violation]) -> bool {
    results.iter().any(|r| r.severity == "violation")
}

/// Run every How check over the contract: native presence/cardinality checks
/// plus the graph cross-reference + trace rules. Returns all violations.
pub fn validate_how(c: &HowContract) -> Vec<Violation> {
    let mut out = Vec::new();
    check_decisions(c, &mut out);
    check_principles(c, &mut out);
    check_patterns(c, &mut out);
    check_app_contract(c, &mut out);
    check_interfaces(c, &mut out);
    check_realisations(c, &mut out);
    out.extend(run_rules(&how_to_turtle(c), how_rules()));
    out
}

/// §4.2 — realisation declarations: known backend, a tier the backend
/// supports, a command for external backends, unique ids.
fn check_realisations(c: &HowContract, out: &mut Vec<Violation>) {
    let mut seen = std::collections::BTreeSet::new();
    for r in &c.realisations {
        if !seen.insert(r.id.clone()) {
            out.push(v(&r.id, "id", "§4.2 Realisation ids must be unique."));
        }
        match r.backend.as_str() {
            "csharp" => {}
            "kotlin" | "plugin" => {
                if r.tier.as_deref() == Some("full") {
                    out.push(v(&r.id, "tier", &format!(
                        "§4.2 Backend '{}' supports only the oracle-only tier (the realiser owns the domain design).", r.backend)));
                }
            }
            other => out.push(v(&r.id, "backend", &format!(
                "§4.2 Unknown backend '{other}' — built-ins: csharp, kotlin; external backends use `backend: plugin` + plugin_cmd."))),
        }
        if let Some(t) = r.tier.as_deref() {
            if t != "full" && t != "oracle-only" {
                out.push(v(&r.id, "tier", &format!("§4.2 Unknown tier '{t}' — full | oracle-only.")));
            }
        }
        if r.backend == "plugin" && r.plugin_cmd.as_deref().map(str::trim).unwrap_or("").is_empty() {
            out.push(v(&r.id, "plugin_cmd",
                "§4.2 An external realisation must carry plugin_cmd (manifest on stdin → file plan on stdout)."));
        }
    }
}

fn check_decisions(c: &HowContract, out: &mut Vec<Violation>) {
    for d in &c.top_decisions {
        if d.rationale.trim().is_empty() {
            out.push(v(&d.id, "rationale",
                "§4.1 A top decision must carry rationale (what it buys, when it applies, when it does not)."));
        }
    }
}

fn check_principles(c: &HowContract, out: &mut Vec<Violation>) {
    for p in &c.principles {
        if p.statement.trim().is_empty() {
            out.push(v(&p.id, "statement",
                "§4.1 A principle must be stated checkably (a vague principle cannot back a verification)."));
        }
    }
}

fn check_patterns(c: &HowContract, out: &mut Vec<Violation>) {
    for p in &c.patterns {
        if p.realizes.is_empty() {
            out.push(v(&p.id, "realizes",
                "§4.1 A pattern must realise at least one principle (it is how a principle is concretely met)."));
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
