//! Validation orchestration — native shape checks, then the SPARQL rules.
//!
//! Splits along the same line as `pf::validate`: presence/cardinality checks
//! (format versions, duplicate ids, the claim-format status ladder) stay
//! native here, while the cross-reference ontology rules (PRD §6 rules 2–5)
//! run as SPARQL over the Turtle projection. Schema faults collected at load
//! time are carried through, so one pass reports the whole store.

use std::collections::BTreeMap;

use product_core::pf::sparql_rules::run_rules;
use product_core::pf::validate::Violation;

use crate::claim::{Claim, ClaimStatus};
use crate::rules::ONTOLOGY_RULES;
use crate::store::DddStore;
use crate::turtle;

/// The entry format versions this tool validates against; each entry is
/// checked against the version it declares (PRD §6).
pub const SUPPORTED_FORMATS: &[u32] = &[1, 2];

/// Run every check over the store; empty means conformant.
pub fn validate_store(store: &DddStore) -> Vec<Violation> {
    let mut out = store.schema_violations.clone();
    out.extend(format_checks(store));
    out.extend(format_gate_checks(store));
    out.extend(duplicate_id_checks(store));
    out.extend(claim_shape_checks(&store.claims));
    out.extend(tolerance_checks(store));
    let mut sparql = run_rules(&turtle::to_turtle(store), ONTOLOGY_RULES);
    for v in &mut sparql {
        v.focus = turtle::decode_ref(&v.focus);
    }
    out.extend(sparql);
    out
}

/// How many of these violations block (warnings do not).
pub fn blocking(violations: &[Violation]) -> usize {
    violations.iter().filter(|v| v.severity != "warning").count()
}

fn fault(focus: &str, path: &str, message: String) -> Violation {
    Violation { focus: focus.to_string(), path: path.to_string(), message, severity: "violation".to_string() }
}

/// Validation is always against the declared format; v1 plus v2 are known.
fn format_checks(store: &DddStore) -> Vec<Violation> {
    let mut out = Vec::new();
    let mut check = |id: &str, format: u32| {
        if !SUPPORTED_FORMATS.contains(&format) {
            out.push(fault(
                id,
                "format",
                format!("declares format {format}; this tool validates formats 1-2"),
            ));
        }
    };
    for p in &store.predicates {
        check(&p.id, p.format);
    }
    for c in &store.claims {
        check(&c.id, c.format);
    }
    for d in &store.decisions {
        check(&d.id, d.format);
    }
    for m in &store.manifests {
        check(&m.name, m.file.format);
    }
    for p in &store.patterns {
        check(&p.id, p.format);
    }
    for s in &store.seams {
        check(&s.id, s.format);
    }
    if let Some(c) = &store.config {
        check("config.yaml", c.format);
    }
    out
}

/// Format-2 features must be declared: an entry using them under format 1
/// is a violation, so v1 entries never change meaning silently (PRD §6).
fn format_gate_checks(store: &DddStore) -> Vec<Violation> {
    let mut out = Vec::new();
    for c in &store.claims {
        if c.format < 2 && c.revalidate_by.is_some() {
            out.push(fault(
                &c.id,
                "revalidate_by",
                "revalidate_by is a format-2 field — declare `format: 2`".to_string(),
            ));
        }
    }
    for d in &store.decisions {
        let pinned = d.based_on.iter().filter(|b| b.pin().is_some()).count();
        if d.format < 2 && pinned > 0 {
            out.push(fault(
                &d.id,
                "based_on",
                "pinned basedOn edges are format 2 — declare `format: 2`".to_string(),
            ));
        }
        if d.format >= 2 && pinned < d.based_on.len() {
            out.push(fault(
                &d.id,
                "based_on",
                "a format-2 decision pins every basedOn edge with the claim's status plus changed date at decision time (rule 6)".to_string(),
            ));
        }
    }
    if let Some(c) = &store.config {
        if c.format < 2 && (c.diff.is_some() || c.detect.is_some()) {
            out.push(fault(
                "config.yaml",
                "format",
                "diff/detect sections are format 2 — declare `format: 2`".to_string(),
            ));
        }
    }
    out
}

/// Ids are stable identities; a duplicate makes `why` ambiguous.
fn duplicate_id_checks(store: &DddStore) -> Vec<Violation> {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    let ids = store
        .predicates
        .iter()
        .map(|p| p.id.as_str())
        .chain(store.claims.iter().map(|c| c.id.as_str()))
        .chain(store.decisions.iter().map(|d| d.id.as_str()))
        .chain(store.patterns.iter().map(|p| p.id.as_str()))
        .chain(store.seams.iter().map(|s| s.id.as_str()));
    for id in ids {
        *seen.entry(id).or_default() += 1;
    }
    seen.iter()
        .filter(|(_, n)| **n > 1)
        .map(|(id, n)| fault(id, "id", format!("id is filed {n} times — ids are stable, never shared")))
        .collect()
}

/// The claim-format status ladder: a live claim keeps its falsifier; any
/// status resting on exercise keeps its evidence (retired keeps what killed it).
fn claim_shape_checks(claims: &[Claim]) -> Vec<Violation> {
    let mut out = Vec::new();
    for c in claims {
        let blank = |o: &Option<String>| o.as_deref().map(str::trim).unwrap_or("").is_empty();
        if c.status != ClaimStatus::Retired && blank(&c.falsifier) {
            out.push(fault(
                &c.id,
                "falsifier",
                format!("a {} claim must state the observation that would kill it", c.status.as_str()),
            ));
        }
        if c.status != ClaimStatus::Projected && blank(&c.evidence) {
            out.push(fault(
                &c.id,
                "evidence",
                format!("a {} claim must state what its status rests on", c.status.as_str()),
            ));
        }
    }
    out
}

/// `predicate-format.md`: without a tolerance the entry is invalid.
fn tolerance_checks(store: &DddStore) -> Vec<Violation> {
    store
        .predicates
        .iter()
        .filter(|p| p.tolerance.trim().is_empty())
        .map(|p| fault(&p.id, "tolerance", "a predicate must declare its tolerance bound".to_string()))
        .collect()
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
